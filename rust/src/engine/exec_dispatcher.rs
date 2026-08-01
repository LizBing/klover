use crate::{
    engine::{
        call::Invocation,
        class_init::ClassInitialization,
        exec_error::{ExecError, ExecResult},
        interpreter::{interpreter::Interpreter, interpreter_frame::InterpreterFrame},
        outcome::{RunOutcome, StepOutcome, ThreadExit},
        resolved_method::ResolvedMethod,
    },
    oops::acc_flags::AccFlags,
    runtime::java_thread::JavaThread,
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
        // Root entry has no Java caller. The launcher, JNI boundary, or test
        // harness has already materialized all arguments in `Invocation`.
        if invocation
            .target
            .method()
            .acc_flags
            .contains(AccFlags::ACC_STATIC)
        {
            ClassInitialization::ensure_initialized(invocation.target.holder(), thread.id())?;
        }

        let frame = Self::build_interpreter_frame(invocation)?;

        thread
            .stack_mut()
            .push_interpreter(frame)
            .map_err(|e| ExecError::Stack(e))?;

        Ok(())
    }

    fn enter_static_call(
        &mut self,
        thread: &mut JavaThread,
        target: ResolvedMethod,
        arg_slots: usize,
    ) -> ExecResult<()> {
        // An internal call has a Java caller, so its arguments have not been
        // materialized yet and still reside on the caller's operand stack.
        ClassInitialization::ensure_initialized(target.holder(), thread.id())?;

        let args = thread
            .stack()
            .current_interpreter()
            .map_err(ExecError::Stack)?
            .peek_top_slots(arg_slots)?;

        let frame = Self::build_interpreter_frame(Invocation { target, args })?;

        thread.stack_mut().push_interpreter_call(frame, arg_slots)
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
            match self.interpreter.execute_one(thread)? {
                StepOutcome::Continue => {}

                StepOutcome::Branch(target) => {
                    thread
                        .stack_mut()
                        .current_interpreter_mut()
                        .map_err(|e| ExecError::Stack(e))?
                        .set_pc(target)?;
                }

                StepOutcome::InvokeStatic { target, arg_slots } => {
                    self.enter_static_call(thread, target, arg_slots)?;
                }

                StepOutcome::Return(value) => {
                    thread.stack_mut().pop();

                    if thread.stack().is_empty() {
                        thread.terminate();

                        return Ok(RunOutcome::Terminated(ThreadExit::Returned(value)));
                    }

                    thread
                        .stack_mut()
                        .current_interpreter_mut()
                        .map_err(|e| ExecError::Stack(e))?
                        .push_return_value(value)?;
                }

                StepOutcome::Throw(exception) => {
                    thread.terminate();

                    return Ok(RunOutcome::Terminated(ThreadExit::UncaughtException(
                        exception,
                    )));
                }
            }
        }

        Ok(RunOutcome::QuantumExpired)
    }
}
