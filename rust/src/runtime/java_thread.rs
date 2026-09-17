use std::ops::{Deref, DerefMut};

use crate::engines::{invocation::Invocation, java_frame::JavaFrame};

struct JavaStack {
    frames: Vec<Box<dyn JavaFrame>>,
}

impl JavaStack {
    fn new() -> Self {
        Self {
            frames: Vec::new(),
        }
    }
}

impl JavaStack {
    fn push(&mut self, frame: impl JavaFrame + 'static) {
        self.frames.push(Box::new(frame));
    }

    fn pop(&mut self) -> Option<Box<dyn JavaFrame>> {
        self.frames.pop()
    }

    fn current_frame(&self) -> Option<&(dyn JavaFrame + 'static)> {
        self.frames.last()
            .map(|f| f.deref())
    }

    fn current_frame_mut(&mut self) -> Option<&mut (dyn JavaFrame + 'static)> {
        self.frames.last_mut()
            .map(|f| f.deref_mut())
    }
}

pub enum JavaThreadState {
    New,
    Runnable,
    Terminated,
}

pub struct JavaThread {
    state: JavaThreadState,
    stack: JavaStack,
}

impl JavaThread {
    pub fn new() -> Self {
        Self {
            state: JavaThreadState::New,
            stack: JavaStack::new(),
        }
    }
}

impl JavaThread {
    pub fn state(&self) -> &JavaThreadState {
        &self.state
    }
}

impl JavaThread {
    // state: New -> Runnable
    pub fn start_with_frame(&mut self, frame: impl JavaFrame + 'static) {
        assert!(matches!(self.state, JavaThreadState::New), "Thread state should be 'New' before getting started.");

        self.stack.push(frame);
        self.state = JavaThreadState::Runnable;
    }
}

impl JavaThread {
    pub fn push_frame(&mut self, frame: impl JavaFrame + 'static) {
        assert!(matches!(self.state, JavaThreadState::Runnable), "Thread state should be runnable.");

        self.stack.push(frame);
    }

    pub fn pop_frame(&mut self) -> Option<Box<dyn JavaFrame>> {
        assert!(matches!(self.state, JavaThreadState::Runnable), "Thread state should be runnable.");

        self.stack.pop()
    }

    pub fn current_frame(&mut self) -> Option<&(dyn JavaFrame + 'static)> {
        self.stack.current_frame()
    }

    pub fn current_frame_mut(&mut self) -> Option<&mut (dyn JavaFrame + 'static)> {
        self.stack.current_frame_mut()
    }
}
