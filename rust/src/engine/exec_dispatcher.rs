use crate::{
    class_loader::ms_api::MSRef,
    engine::{
        call::Invocation,
        class_init::{
            ClassInitFrame, ClassInitPhase, ClassInitialization, Continuation,
        },
        exec_error::{ExecError, ExecResult},
        interpreter::{interpreter::Interpreter, interpreter_frame::InterpreterFrame},
        outcome::{RetValue, RunOutcome, StepOutcome, ThreadExit},
        resolved_method::ResolvedMethod,
    },
    oops::{
        acc_flags::AccFlags,
        cp_entry::ResolvedFieldRef,
        normal_klass::{ClassInitAction, NormalKlass},
    },
    runtime::{
        java_stack::JavaFrame,
        java_thread::JavaThread,
    },
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
            self.request_class_initialization(
                thread,
                holder,
                Continuation::EnterRoot(invocation),
            )
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

    fn commit_root(
        &mut self,
        thread: &mut JavaThread,
        invocation: Invocation,
    ) -> ExecResult<()> {
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
            ClassInitAction::Initialize => {
                if let Err(error) = ClassInitialization::prepare_claimed(&klass) {
                    let _ = ClassInitialization::abandon(&klass, thread.id());
                    return Err(error);
                }

                let frame = ClassInitFrame::new_claimed(klass.clone(), continuation);
                if let Err(error) = thread.stack_mut().push_class_init(frame) {
                    ClassInitialization::abandon(&klass, thread.id())?;
                    return Err(ExecError::Stack(error));
                }

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
            Continuation::ResumeCaller => Ok(()),
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
            ClassInitPhase::InitializeSuper => {
                thread
                    .stack_mut()
                    .current_class_init_mut()
                    .map_err(ExecError::Stack)?
                    .set_phase(ClassInitPhase::InvokeClinit);

                if !klass.is_interface() {
                    if let Some(super_klass) = klass.super_klass_ref() {
                        self.request_class_initialization(
                            thread,
                            super_klass,
                            Continuation::ResumeCaller,
                        )?;
                    }
                }
                Ok(())
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

            ClassInitPhase::AwaitClinit => {
                Err(ExecError::InvalidClassInitializationFrameState)
            }

            ClassInitPhase::Complete => {
                ClassInitialization::complete(&klass, thread.id())?;
                let frame = thread
                    .stack_mut()
                    .pop()
                    .ok_or(ExecError::NoCurrentFrame)?;
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
        let frame = thread
            .stack_mut()
            .pop()
            .ok_or(ExecError::NoCurrentFrame)?;
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
        exception: crate::engine::outcome::PendingException,
    ) -> ExecResult<RunOutcome> {
        while let Some(frame) = thread.stack_mut().pop() {
            if let JavaFrame::ClassInit(frame) = frame {
                let (klass, _) = frame.into_parts();
                ClassInitialization::fail(&klass, thread.id(), exception.clone())?;
            }
        }

        thread.terminate();
        Ok(RunOutcome::Terminated(ThreadExit::UncaughtException(
            exception,
        )))
    }

    /// Shared interpreter-frame construction after arguments are materialized.
    fn build_interpreter_frame(invocation: Invocation) -> ExecResult<InterpreterFrame> {
        InterpreterFrame::new(invocation.target, &invocation.args)
    }
}

impl ExecDispatcher {
    pub fn run_quantum(
        &mut self,
        thread: &mut JavaThread,
        budget: usize,
    ) -> ExecResult<RunOutcome> {
        for _ in 0..budget {
            if thread.stack().current_is_class_init() {
                self.advance_class_initialization(thread)?;
                continue;
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
                    if let Some(outcome) = self.complete_interpreter_return(thread, value)? {
                        return Ok(outcome);
                    }
                }

                StepOutcome::Throw(exception) => {
                    return self.terminate_with_exception(thread, exception);
                }
            }
        }

        Ok(RunOutcome::QuantumExpired)
    }
}
