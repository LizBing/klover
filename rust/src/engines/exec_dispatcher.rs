use crate::{
    engines::{
        exec_error::{ExecError, ExecErrorKind, ExecResult},
        interpreter::interpreter_frame::InterpreterFrame,
        invocation::Invocation,
        protocol::{DispatchOutcome, EngineExit, ExecBudget},
    },
    runtime::java_thread::JavaThread,
};

pub struct ExecDispatcher;

impl ExecDispatcher {
    pub fn start<'exec, 'meta: 'exec>(
        thread: &mut JavaThread<'exec, 'meta>,
        invocation: Invocation<'exec, 'meta>,
    ) -> ExecResult<()> {
        // directly create Interpreter frame for now
        let frame = InterpreterFrame::try_new(invocation)?;
        thread.start_with_frame(frame);

        Ok(())
    }

    pub fn poll<'exec, 'meta: 'exec>(
        thread: &mut JavaThread<'exec, 'meta>,
        budget: &mut ExecBudget,
    ) -> ExecResult<DispatchOutcome> {
        loop {
            if thread.is_terminated() {
                return Err(ExecError::new(ExecErrorKind::ThreadTerminated));
            }

            let Some(frame) = thread.current_frame_mut() else {
                return Err(ExecError::new(ExecErrorKind::NoFrame));
            };

            // The exit borrows metadata, not the temporary mutable frame borrow.
            // End that borrow before changing the thread's frame stack.
            let exit = frame.resume(budget)
                .map_err(|e| {
                    thread.terminate();
                    e
                })?;
            match exit {
                EngineExit::BudgetExhausted => return Ok(DispatchOutcome::Yielded),
                EngineExit::Return(r) => {
                    thread.pop_frame();
                    match thread.current_frame_mut() {
                        Some(frame) => frame.accept_return(r)
                            .map_err(|e| {
                                thread.terminate();
                                e
                            })?,
                        None => return Ok(DispatchOutcome::Completed(r)),
                    }
                }
                EngineExit::Invoke(invocation) => {
                    let frame = InterpreterFrame::try_new(invocation)
                        .map_err(|e| {
                            thread.terminate();
                            e
                        })?;
                    thread.push_frame(frame);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/exec_dispatcher.rs"]
mod tests;
