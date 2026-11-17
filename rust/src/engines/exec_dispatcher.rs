use crate::{
    engines::{
        exec_error::{ExecError, ExecErrorKind, ExecResult},
        initialization_frame::InitializationFrame,
        interpreter::interpreter_frame::InterpreterFrame,
        invocation::Invocation,
        protocol::{DispatchOutcome, EngineExit, ExecBudget, ExecutionContext},
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

            let context = ExecutionContext {
                thread_id: thread.id(),
            };
            let Some(frame) = thread.current_frame_mut() else {
                return Err(ExecError::new(ExecErrorKind::NoFrame));
            };

            // The exit borrows metadata, not the temporary mutable frame borrow.
            // End that borrow before changing the thread's frame stack.
            let exit = frame.resume(budget, &context).map_err(|e| {
                thread.fail(&e);
                e
            })?;
            match exit {
                EngineExit::Initialize(klass) => {
                    thread.push_frame(InitializationFrame::new(klass));
                }
                EngineExit::InitializationComplete => {
                    thread.pop_frame();
                    if thread.is_terminated() {
                        return Ok(DispatchOutcome::Completed(
                            crate::engines::protocol::MethodReturn::Void,
                        ));
                    }
                }
                EngineExit::BlockedOnInitialization => {
                    return Ok(DispatchOutcome::BlockedOnInitialization);
                }
                EngineExit::BudgetExhausted => return Ok(DispatchOutcome::Yielded),
                EngineExit::Return(r) => {
                    thread.pop_frame();
                    match thread.current_frame_mut() {
                        Some(frame) => frame.accept_return(r).map_err(|e| {
                            thread.fail(&e);
                            e
                        })?,
                        None => return Ok(DispatchOutcome::Completed(r)),
                    }
                }
                EngineExit::Invoke(invocation) => {
                    let frame = InterpreterFrame::try_new(invocation).map_err(|e| {
                        thread.fail(&e);
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
