use crate::{engine::{call::Invocation, engine_error::{ExecError, ExecResult}, interpreter::{interpreter::Interpreter, interpreter_frame::InterpreterFrame}, outcome::{RunOutcome, StepOutcome, ThreadExit}, resolved_method::ResolvedMethod}, runtime::java_thread::JavaThread};

#[derive(Debug)]
pub struct ExecDispatcher {
    interpreter: Interpreter
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
        let frame = InterpreterFrame::new(
            invocation.target,
            &invocation.args,
        )?;

        thread.stack_mut().push_interpreter(frame)
            .map_err(|e| ExecError::Stack(e))?;

        Ok(())
    }

    fn enter_call(
        &mut self,
        thread: &mut JavaThread,
        target: ResolvedMethod,
        arg_slots: usize,
    ) -> ExecResult<()> {
        let args = thread
            .stack_mut()
            .current_interpreter_mut()
            .map_err(|e| ExecError::Stack(e))?
            .take_top_slots(arg_slots)?;

        let frame = InterpreterFrame::new(target, &args)?;
        thread.stack_mut().push_interpreter(frame)
            .map_err(|e| ExecError::Stack(e))?;

        Ok(())
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

                StepOutcome::Call {
                    target,
                    arg_slots,
                } => {
                    self.enter_call(
                        thread,
                        target,
                        arg_slots,
                    )?;
                }

                StepOutcome::Return(value) => {
                    thread.stack_mut().pop();

                    if thread.stack().is_empty() {
                        thread.terminate();

                        return Ok(RunOutcome::Terminated(
                            ThreadExit::Returned(value),
                        ));
                    }

                    thread
                        .stack_mut()
                        .current_interpreter_mut()
                        .map_err(|e| ExecError::Stack(e))?
                        .push_return_value(value)?;
                }

                StepOutcome::Throw(exception) => {
                    thread.terminate();
                
                    return Ok(RunOutcome::Terminated(
                        ThreadExit::UncaughtException(exception),
                    ));
                }
            }
        }

        Ok(RunOutcome::QuantumExpired)
    }
}
