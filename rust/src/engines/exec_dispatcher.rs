use crate::{
    engines::{
        exec_error::{ExecError, ExecErrorKind, ExecResult},
        interpreter::interpreter_frame::InterpreterFrame,
        invocation::Invocation, protocol::{DispatchOutcome, EngineExit, ExecBudget},
    },
    runtime::java_thread::JavaThread,
};

pub struct ExecDispatcher;

impl ExecDispatcher {
    pub fn start<'metadata>(thread: &mut JavaThread<'metadata>, invocation: Invocation<'metadata>) -> ExecResult<()> {
        // directly create Interpreter frame for now
        let frame = InterpreterFrame::new(invocation);
        thread.start_with_frame(frame);

        Ok(())
    }

    pub fn poll(thread: &mut JavaThread, budget: &mut ExecBudget) -> ExecResult<DispatchOutcome> {
        let Some(frame) = thread.current_frame_mut() else {
            return Err(ExecError::new(ExecErrorKind::NoFrame));
        };

        match frame.resume(budget)? {
            EngineExit::BudgetExhausted => Ok(DispatchOutcome::Yielded),
            EngineExit::Return(r) => Ok(DispatchOutcome::Completed(r)),
        }
    }
}
