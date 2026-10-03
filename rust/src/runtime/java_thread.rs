use std::ops::{Deref, DerefMut};

use crate::engines::java_frame::JavaFrame;

struct JavaStack<'metadata> {
    frames: Vec<Box<dyn JavaFrame + 'metadata>>,
}

impl<'metadata> JavaStack<'metadata> {
    fn new() -> Self {
        Self { frames: Vec::new() }
    }
}

impl<'metadata> JavaStack<'metadata> {
    fn push(&mut self, frame: impl JavaFrame + 'metadata) {
        self.frames.push(Box::new(frame));
    }

    fn pop(&mut self) -> Option<Box<dyn JavaFrame + 'metadata>> {
        self.frames.pop()
    }

    fn current_frame(&self) -> Option<&(dyn JavaFrame + 'metadata)> {
        self.frames.last().map(|f| f.deref())
    }

    fn current_frame_mut(&mut self) -> Option<&mut (dyn JavaFrame + 'metadata)> {
        self.frames.last_mut().map(|f| f.deref_mut())
    }
}

pub enum JavaThreadState {
    New,
    Runnable,
    Terminated,
}

/// Frames may borrow class metadata, which must outlive the thread's stack.
/// The lifetime on the trait objects is explicit: Box<dyn JavaFrame> alone
/// would require every frame to own its data or borrow it for 'static.
pub struct JavaThread<'metadata> {
    state: JavaThreadState,
    stack: JavaStack<'metadata>,
}

impl<'metadata> JavaThread<'metadata> {
    pub fn new() -> Self {
        Self {
            state: JavaThreadState::New,
            stack: JavaStack::new(),
        }
    }
}

impl<'metadata> JavaThread<'metadata> {
    pub fn state(&self) -> &JavaThreadState {
        &self.state
    }
}

impl<'metadata> JavaThread<'metadata> {
    // state: New -> Runnable
    pub fn start_with_frame(&mut self, frame: impl JavaFrame + 'metadata) {
        assert!(
            matches!(self.state, JavaThreadState::New),
            "Thread state should be 'New' before getting started."
        );

        self.stack.push(frame);
        self.state = JavaThreadState::Runnable;
    }
}

impl<'metadata> JavaThread<'metadata> {
    pub fn push_frame(&mut self, frame: impl JavaFrame + 'metadata) {
        assert!(
            matches!(self.state, JavaThreadState::Runnable),
            "Thread state should be runnable."
        );

        self.stack.push(frame);
    }

    pub fn pop_frame(&mut self) -> Option<Box<dyn JavaFrame + 'metadata>> {
        assert!(
            matches!(self.state, JavaThreadState::Runnable),
            "Thread state should be runnable."
        );

        self.stack.pop()
    }

    pub fn current_frame(&mut self) -> Option<&(dyn JavaFrame + 'metadata)> {
        self.stack.current_frame()
    }

    pub fn current_frame_mut(&mut self) -> Option<&mut (dyn JavaFrame + 'metadata)> {
        self.stack.current_frame_mut()
    }
}
