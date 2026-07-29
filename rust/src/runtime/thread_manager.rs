use crate::runtime::{java_thread::{JavaThread, JavaThreadID}, runtime_error::{ThreadError, ThreadResult}};

#[derive(Debug)]
pub struct ThreadManager {
    next_id: u64,
    stack_limit: usize,
}

impl ThreadManager {
    pub fn new(stack_limit: usize) -> Self {
        Self {
            next_id: 1,
            stack_limit,
        }
    }

    pub fn create_thread(
        &mut self,
    ) -> ThreadResult<JavaThread> {
        let raw_id = self.next_id;

        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(ThreadError::IDExhausted)?;

        Ok(JavaThread::new(
            JavaThreadID::new(raw_id)?,
            self.stack_limit,
        ))
    }
}
