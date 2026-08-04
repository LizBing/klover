use crate::{
    class_loader::ms_api::MSRef,
    engine::{
        call::Invocation,
        class_init::{ClassInitFrame, ClassInitPhase, ClassInitialization, Continuation},
        exec_error::{ExecError, ExecResult, JavaExceptionKind},
        interpreter::{interpreter::Interpreter, interpreter_frame::InterpreterFrame},
        outcome::{PendingException, RetValue, RunOutcome, StepOutcome, ThreadExit},
        resolved_method::ResolvedMethod,
    },
    oops::{
        acc_flags::AccFlags,
        cp_entry::ResolvedFieldRef,
        normal_klass::{ClassInitAction, NormalKlass},
    },
    runtime::{java_stack::JavaFrame, java_thread::JavaThread},
};

#[derive(Debug)]
pub struct ExecDispatcher {
    interpreter: Interpreter,
}

impl ExecDispatcher {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter,
        }
    }

    pub fn enter_root(
        &mut self,
        thread: &mut JavaThread,
        invocation: Invocation,
    ) -> ExecResult<()> {
        // Root entry has no Java caller. Keep the materialized Invocation in
        // the continuation while its declaring class initializes.
        if invocation
            .target
            .method()
            .acc_flags
            .contains(AccFlags::ACC_STATIC)
        {
            let holder = invocation.target.holder_ref();
            self.request_class_initialization(thread, holder, Continuation::EnterRoot(invocation))
        } else {
            self.commit_root(thread, invocation)
        }
    }

    fn request_static_call(
        &mut self,
        thread: &mut JavaThread,
        target: ResolvedMethod,
        arg_slots: usize,
    ) -> ExecResult<()> {
        let holder = target.holder_ref();
        self.request_class_initialization(
            thread,
            holder,
            Continuation::InvokeStatic { target, arg_slots },
        )
    }

    fn commit_static_call(
        &mut self,
        thread: &mut JavaThread,
        target: ResolvedMethod,
        arg_slots: usize,
    ) -> ExecResult<()> {
        // Arguments remain on the suspended caller until initialization has
        // succeeded and this commit path is reached.
        let args = thread
            .stack()
            .current_interpreter()
            .map_err(ExecError::Stack)?
            .peek_top_slots(arg_slots)?;

        let frame = Self::build_interpreter_frame(Invocation { target, args })?;
        thread.stack_mut().push_interpreter_call(frame, arg_slots)
    }

    fn request_get_static(
        &mut self,
        thread: &mut JavaThread,
        resolved: ResolvedFieldRef,
    ) -> ExecResult<()> {
        self.request_class_initialization(
            thread,
            resolved.holder.clone(),
            Continuation::GetStatic(resolved),
        )
    }

    fn commit_get_static(
        &mut self,
        thread: &mut JavaThread,
        resolved: ResolvedFieldRef,
    ) -> ExecResult<()> {
        let slots = resolved.holder.read_static_field(&resolved.field)?;
        thread
            .stack_mut()
            .current_interpreter_mut()
            .map_err(ExecError::Stack)?
            .push_slots(&slots)
    }

    fn request_put_static(
        &mut self,
        thread: &mut JavaThread,
        resolved: ResolvedFieldRef,
    ) -> ExecResult<()> {
        self.request_class_initialization(
            thread,
            resolved.holder.clone(),
            Continuation::PutStatic(resolved),
        )
    }

    fn commit_put_static(
        &mut self,
        thread: &mut JavaThread,
        resolved: ResolvedFieldRef,
    ) -> ExecResult<()> {
        let slot_count = resolved.field.desc.slot_count();
        let slots = thread
            .stack()
            .current_interpreter()
            .map_err(ExecError::Stack)?
            .peek_top_slots(slot_count)?;

        resolved
            .holder
            .write_static_field(&resolved.field, &slots)?;

        thread
            .stack_mut()
            .current_interpreter_mut()
            .map_err(ExecError::Stack)?
            .drop_top_slots(slot_count)
    }

    fn commit_root(&mut self, thread: &mut JavaThread, invocation: Invocation) -> ExecResult<()> {
        let frame = Self::build_interpreter_frame(invocation)?;
        thread
            .stack_mut()
            .push_interpreter(frame)
            .map_err(ExecError::Stack)
    }

    fn request_class_initialization(
        &mut self,
        thread: &mut JavaThread,
        klass: MSRef<NormalKlass>,
        continuation: Continuation,
    ) -> ExecResult<()> {
        match ClassInitialization::begin(&klass, thread.id())? {
            ClassInitAction::AlreadyInitialized | ClassInitAction::RecursiveRequest => {
                self.apply_continuation(thread, continuation)
            }
            ClassInitAction::Claimed => {
                let prerequisites = ClassInitialization::prerequisites(&klass);
                let frame = ClassInitFrame::new_claimed(klass.clone(), prerequisites, continuation);
                if let Err(error) = thread.stack_mut().push_class_init(frame) {
                    ClassInitialization::abort(&klass, thread.id())?;
                    return Err(ExecError::Stack(error));
                }

                Ok(())
            }
            ClassInitAction::Erroneous => {
                thread.pending_exception = Some(PendingException::JVMGen(
                    JavaExceptionKind::NoClassDefFoundError,
                ));
                Ok(())
            }
        }
    }

    fn apply_continuation(
        &mut self,
        thread: &mut JavaThread,
        continuation: Continuation,
    ) -> ExecResult<()> {
        match continuation {
            Continuation::EnterRoot(invocation) => self.commit_root(thread, invocation),
            Continuation::InvokeStatic { target, arg_slots } => {
                self.commit_static_call(thread, target, arg_slots)
            }
            Continuation::GetStatic(resolved) => self.commit_get_static(thread, resolved),
            Continuation::PutStatic(resolved) => self.commit_put_static(thread, resolved),
            Continuation::ResumeInitializer => {
                let initializer = thread
                    .stack_mut()
                    .current_class_init_mut()
                    .map_err(ExecError::Stack)?;
                if initializer.phase() != ClassInitPhase::AwaitPrerequisite {
                    return Err(ExecError::InvalidClassInitializationFrameState);
                }
                initializer.set_phase(ClassInitPhase::InitializePrerequisites);
                Ok(())
            }
        }
    }

    fn advance_class_initialization(&mut self, thread: &mut JavaThread) -> ExecResult<()> {
        let (klass, phase) = {
            let frame = thread
                .stack()
                .current_class_init()
                .map_err(ExecError::Stack)?;
            (frame.klass().clone(), frame.phase())
        };

        match phase {
            ClassInitPhase::InstallConstantValues => {
                if let Err(error) = ClassInitialization::install_constant_values(&klass) {
                    let frame = thread.stack_mut().pop().ok_or(ExecError::NoCurrentFrame)?;
                    if !matches!(frame, JavaFrame::ClassInit(_)) {
                        return Err(ExecError::InvalidClassInitializationFrameState);
                    }
                    ClassInitialization::abort(&klass, thread.id())?;
                    return Err(error);
                }

                thread
                    .stack_mut()
                    .current_class_init_mut()
                    .map_err(ExecError::Stack)?
                    .set_phase(ClassInitPhase::InitializePrerequisites);
                Ok(())
            }

            ClassInitPhase::InitializePrerequisites => {
                let prerequisite = thread
                    .stack_mut()
                    .current_class_init_mut()
                    .map_err(ExecError::Stack)?
                    .next_prerequisite();

                let Some(prerequisite) = prerequisite else {
                    thread
                        .stack_mut()
                        .current_class_init_mut()
                        .map_err(ExecError::Stack)?
                        .set_phase(ClassInitPhase::InvokeClinit);
                    return Ok(());
                };

                thread
                    .stack_mut()
                    .current_class_init_mut()
                    .map_err(ExecError::Stack)?
                    .set_phase(ClassInitPhase::AwaitPrerequisite);
                self.request_class_initialization(
                    thread,
                    prerequisite,
                    Continuation::ResumeInitializer,
                )
            }

            ClassInitPhase::AwaitPrerequisite => {
                Err(ExecError::InvalidClassInitializationFrameState)
            }

            ClassInitPhase::InvokeClinit => {
                let Some(method) = klass.find_declared_method("<clinit>", "()V") else {
                    thread
                        .stack_mut()
                        .current_class_init_mut()
                        .map_err(ExecError::Stack)?
                        .set_phase(ClassInitPhase::Complete);
                    return Ok(());
                };

                let target = ResolvedMethod::new(klass.clone(), method);
                let clinit = Self::build_interpreter_frame(Invocation {
                    target,
                    args: Vec::new(),
                })?;

                thread
                    .stack_mut()
                    .current_class_init_mut()
                    .map_err(ExecError::Stack)?
                    .set_phase(ClassInitPhase::AwaitClinit);

                thread
                    .stack_mut()
                    .push_interpreter(clinit)
                    .map_err(ExecError::Stack)
            }

            ClassInitPhase::AwaitClinit => Err(ExecError::InvalidClassInitializationFrameState),

            ClassInitPhase::Complete => {
                ClassInitialization::complete(&klass, thread.id())?;
                let frame = thread.stack_mut().pop().ok_or(ExecError::NoCurrentFrame)?;
                let JavaFrame::ClassInit(frame) = frame else {
                    return Err(ExecError::InvalidClassInitializationFrameState);
                };
                let (_, continuation) = frame.into_parts();
                self.apply_continuation(thread, continuation)
            }
        }
    }

    fn complete_interpreter_return(
        &mut self,
        thread: &mut JavaThread,
        value: RetValue,
    ) -> ExecResult<Option<RunOutcome>> {
        let frame = thread.stack_mut().pop().ok_or(ExecError::NoCurrentFrame)?;
        if !matches!(frame, JavaFrame::Interpreter(_)) {
            return Err(ExecError::InvalidClassInitializationFrameState);
        }

        if thread.stack().is_empty() {
            thread.terminate();
            return Ok(Some(RunOutcome::Terminated(ThreadExit::Returned(value))));
        }

        if thread.stack().current_is_class_init() {
            if !matches!(value, RetValue::Void) {
                return Err(ExecError::InvalidClassInitializerReturn);
            }

            let init = thread
                .stack_mut()
                .current_class_init_mut()
                .map_err(ExecError::Stack)?;
            if init.phase() != ClassInitPhase::AwaitClinit {
                return Err(ExecError::InvalidClassInitializationFrameState);
            }
            init.set_phase(ClassInitPhase::Complete);
            return Ok(None);
        }

        thread
            .stack_mut()
            .current_interpreter_mut()
            .map_err(ExecError::Stack)?
            .push_return_value(value)?;
        Ok(None)
    }

    fn terminate_with_exception(
        &mut self,
        thread: &mut JavaThread,
        exception: PendingException,
    ) -> ExecResult<RunOutcome> {
        let mut cleanup_error = None;
        while let Some(frame) = thread.stack_mut().pop() {
            if let JavaFrame::ClassInit(frame) = frame {
                let phase = frame.phase();
                let (klass, _) = frame.into_parts();
                let result = match phase {
                    // An exception from this class's own <clinit>, or from one
                    // of its initialization prerequisites, makes it erroneous.
                    // ExceptionInInitializerError wrapping is deferred until
                    // Java exception objects and type checks are available.
                    ClassInitPhase::AwaitClinit | ClassInitPhase::AwaitPrerequisite => {
                        ClassInitialization::fail(&klass, thread.id())
                    }
                    // No Java code should be able to throw in any other phase.
                    // Release the claim so an engine bug cannot wedge the class.
                    _ => {
                        let result = ClassInitialization::abort(&klass, thread.id());
                        if result.is_ok() {
                            cleanup_error = Some(ExecError::InvalidClassInitializationFrameState);
                        }
                        result
                    }
                };
                if cleanup_error.is_none() {
                    cleanup_error = result.err();
                }
            }
        }

        if let Some(error) = cleanup_error {
            return Err(error);
        }

        thread.terminate();
        Ok(RunOutcome::Terminated(ThreadExit::UncaughtException(
            exception,
        )))
    }

    /// An ExecError is a VM failure rather than a Java exception. The current
    /// run cannot resume, so discard its frames and release every live claim.
    fn abort_after_engine_error(&mut self, thread: &mut JavaThread) {
        while let Some(frame) = thread.stack_mut().pop() {
            if let JavaFrame::ClassInit(frame) = frame {
                let (klass, _) = frame.into_parts();
                let _ = ClassInitialization::abort(&klass, thread.id());
            }
        }
    }

    /// Shared interpreter-frame construction after arguments are materialized.
    fn build_interpreter_frame(invocation: Invocation) -> ExecResult<InterpreterFrame> {
        InterpreterFrame::new(invocation.target, &invocation.args)
    }
}

impl ExecDispatcher {
    fn run_one(&mut self, thread: &mut JavaThread) -> ExecResult<Option<RunOutcome>> {
        if thread.stack().current_is_class_init() {
            self.advance_class_initialization(thread)?;
            return Ok(None);
        }

        match self.interpreter.execute_one(thread)? {
            StepOutcome::Continue => {}

            StepOutcome::Branch(target) => {
                thread
                    .stack_mut()
                    .current_interpreter_mut()
                    .map_err(ExecError::Stack)?
                    .set_pc(target)?;
            }

            StepOutcome::GetStatic(resolved) => {
                self.request_get_static(thread, resolved)?;
            }

            StepOutcome::PutStatic(resolved) => {
                self.request_put_static(thread, resolved)?;
            }

            StepOutcome::InvokeStatic { target, arg_slots } => {
                self.request_static_call(thread, target, arg_slots)?;
            }

            StepOutcome::Return(value) => {
                return self.complete_interpreter_return(thread, value);
            }

            StepOutcome::Throw(exception) => {
                return self.terminate_with_exception(thread, exception).map(Some);
            }
        }

        Ok(None)
    }

    pub fn run_quantum(
        &mut self,
        thread: &mut JavaThread,
        budget: usize,
    ) -> ExecResult<RunOutcome> {
        for _ in 0..budget {
            if let Some(exception) = thread.pending_exception.take() {
                return self.terminate_with_exception(thread, exception);
            }

            match self.run_one(thread) {
                Ok(Some(outcome)) => {
                    return Ok(outcome);
                }
                Ok(None) => {}
                Err(error) => {
                    self.abort_after_engine_error(thread);
                    return Err(error);
                }
            }
        }

        Ok(RunOutcome::QuantumExpired)
    }
}
