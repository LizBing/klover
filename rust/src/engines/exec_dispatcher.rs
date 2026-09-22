use crate::{
    engines::{
        exec_error::{ExecError, ExecErrorKind, ExecResult},
        interpreter::interpreter_frame::InterpreterFrame,
        invocation::Invocation,
    },
    oops::jvalue::JValue,
    runtime::java_thread::JavaThread,
};

pub struct ExecBudget {
    remaining: u64,
}

impl ExecBudget {
    pub fn new(fuel: u64) -> Self {
        Self { remaining: fuel }
    }
}

impl ExecBudget {
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    pub fn is_startable(&self) -> bool {
        self.remaining > 0
    }

    pub fn charge(&mut self, cost: u64) {
        self.remaining = self.remaining.saturating_sub(cost);
    }
}

pub enum MethodReturn {
    Void,
    Value(JValue),
}

// pub struct JavaException;

pub enum EngineExit {
    BudgetExhausted,
    // Invoke(Invocation),
    Return(MethodReturn),
    // Throw(JavaException),
}

pub enum DispatchOutcome {
    Yielded,
    Completed(MethodReturn),
    // Uncaught,
    // Blocked,
}

pub struct ExecDispatcher;

impl ExecDispatcher {
    pub fn start(thread: &mut JavaThread, invocation: Invocation) -> ExecResult<()> {
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
