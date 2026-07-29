use crate::engine::outcome::PendingException;
use crate::runtime::java_stack::JavaStack;
use crate::runtime::runtime_error::{ThreadError, ThreadResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaThreadState {
    New,
    Runnable,
    Terminated,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JavaThreadID(u64);

impl JavaThreadID {
    pub fn new(value: u64) -> ThreadResult<Self> {
        if value == 0 {
            return Err(ThreadError::InvalidID);
        }

        Ok(Self(value))
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

pub struct JavaThread {
    pub id: JavaThreadID,
    pub state: JavaThreadState,
    
    pub stack: JavaStack,
    pub pending_exception: Option<PendingException>,

    pub interrupted: bool
}

impl JavaThread {
    pub fn new(
        id: JavaThreadID,
        stack_limit: usize,
    ) -> Self {
        Self {
            id,
            state: JavaThreadState::New,
            stack: JavaStack::new(stack_limit),
            pending_exception: None,
            interrupted: false,
        }
    }

    pub fn id(&self) -> JavaThreadID {
        self.id
    }

    pub fn state(&self) -> JavaThreadState {
        self.state
    }

    pub fn stack(&self) -> &JavaStack {
        &self.stack
    }

    pub fn stack_mut(&mut self) -> &mut JavaStack {
        &mut self.stack
    }

    pub fn start(&mut self) -> ThreadResult<()> {
        if self.state != JavaThreadState::New {
            return Err(ThreadError::AlreadyStarted);
        }

        self.state = JavaThreadState::Runnable;
        Ok(())
    }

    pub fn terminate(&mut self) {
        self.state = JavaThreadState::Terminated;
    }
}
