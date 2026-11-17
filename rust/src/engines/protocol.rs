use crate::{engines::invocation::Invocation, oops::jvalue::JValue};
use crate::{oops::normal_klass::NormalKlass, runtime::java_thread::JavaThreadId};

pub struct ExecutionContext {
    pub thread_id: JavaThreadId,
}

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

pub enum EngineExit<'exec, 'meta: 'exec> {
    BudgetExhausted,
    Initialize(&'exec NormalKlass<'meta>),
    InitializationComplete,
    BlockedOnInitialization,
    Invoke(Invocation<'exec, 'meta>),
    Return(MethodReturn),
    // Throw(JavaException),
}

pub enum DispatchOutcome {
    Yielded,
    BlockedOnInitialization,
    Completed(MethodReturn),
    // Uncaught,
    // Blocked,
}
