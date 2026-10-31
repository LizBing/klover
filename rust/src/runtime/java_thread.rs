use std::ops::{Deref, DerefMut};

use crate::engines::java_frame::JavaFrame;

struct JavaStack<'exec> {
    frames: Vec<Box<dyn JavaFrame + 'exec>>,
}

impl<'exec> JavaStack<'exec> {
    fn new() -> Self {
        Self { frames: Vec::new() }
    }
}

impl<'exec> JavaStack<'exec> {
    fn push(&mut self, frame: impl JavaFrame + 'exec) {
        self.frames.push(Box::new(frame));
    }

    fn pop(&mut self) -> Option<Box<dyn JavaFrame + 'exec>> {
        self.frames.pop()
    }

    fn current_frame(&self) -> Option<&(dyn JavaFrame + 'exec)> {
        self.frames.last().map(|f| f.deref())
    }

    fn current_frame_mut(&mut self) -> Option<&mut (dyn JavaFrame + 'exec)> {
        self.frames.last_mut().map(|f| f.deref_mut())
    }
}

pub enum JavaThreadState {
    New,
    Runnable,
    Terminated,
}

/// Frames borrow metadata for an execution scope that outlives the thread's stack.
/// The lifetime on the trait objects is explicit: Box<dyn JavaFrame> alone
/// would require every frame to own its data or borrow it for 'static.
pub struct JavaThread<'exec> {
    state: JavaThreadState,
    stack: JavaStack<'exec>,
}

impl<'exec> JavaThread<'exec> {
    pub fn new() -> Self {
        Self {
            state: JavaThreadState::New,
            stack: JavaStack::new(),
        }
    }
}

impl<'exec> JavaThread<'exec> {
    pub fn state(&self) -> &JavaThreadState {
        &self.state
    }
}

impl<'exec> JavaThread<'exec> {
    // state: New -> Runnable
    pub fn start_with_frame(&mut self, frame: impl JavaFrame + 'exec) {
        assert!(
            matches!(self.state, JavaThreadState::New),
            "Thread state should be 'New' before getting started."
        );

        self.stack.push(frame);
        self.state = JavaThreadState::Runnable;
    }
}

impl<'exec> JavaThread<'exec> {
    pub fn push_frame(&mut self, frame: impl JavaFrame + 'exec) {
        assert!(
            matches!(self.state, JavaThreadState::Runnable),
            "Thread state should be runnable."
        );

        self.stack.push(frame);
    }

    pub fn pop_frame(&mut self) -> Option<Box<dyn JavaFrame + 'exec>> {
        assert!(
            matches!(self.state, JavaThreadState::Runnable),
            "Thread state should be runnable."
        );

        self.stack.pop()
    }

    pub fn current_frame(&mut self) -> Option<&(dyn JavaFrame + 'exec)> {
        self.stack.current_frame()
    }

    pub fn current_frame_mut(&mut self) -> Option<&mut (dyn JavaFrame + 'exec)> {
        self.stack.current_frame_mut()
    }
}
