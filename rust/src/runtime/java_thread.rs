use std::ops::{Deref, DerefMut};

use crate::engines::java_frame::JavaFrame;

struct JavaStack<'exec, 'meta: 'exec> {
    frames: Vec<Box<dyn JavaFrame<'exec, 'meta> + 'exec>>,
}

impl<'exec, 'meta: 'exec> JavaStack<'exec, 'meta> {
    fn new() -> Self {
        Self { frames: Vec::new() }
    }
}

impl<'exec, 'meta: 'exec> JavaStack<'exec, 'meta> {
    fn push(&mut self, frame: impl JavaFrame<'exec, 'meta> + 'exec) {
        self.frames.push(Box::new(frame));
    }

    fn pop(&mut self) -> Option<Box<dyn JavaFrame<'exec, 'meta> + 'exec>> {
        self.frames.pop()
    }

    fn current_frame(&self) -> Option<&(dyn JavaFrame<'exec, 'meta> + 'exec)> {
        self.frames.last().map(|f| f.deref())
    }

    fn current_frame_mut(&mut self) -> Option<&mut (dyn JavaFrame<'exec, 'meta> + 'exec)> {
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
pub struct JavaThread<'exec, 'meta: 'exec> {
    state: JavaThreadState,
    stack: JavaStack<'exec, 'meta>,
}

impl<'exec, 'meta: 'exec> JavaThread<'exec, 'meta> {
    pub fn new() -> Self {
        Self {
            state: JavaThreadState::New,
            stack: JavaStack::new(),
        }
    }
}

impl<'exec, 'meta: 'exec> JavaThread<'exec, 'meta> {
    pub fn is_runnable(&self) -> bool {
        matches!(self.state, JavaThreadState::Runnable)
    }

    pub fn is_new(&self) -> bool {
        matches!(self.state, JavaThreadState::New)
    }

    pub fn is_terminated(&self) -> bool {
        matches!(self.state, JavaThreadState::Terminated)
    }
}

impl<'exec, 'meta: 'exec> JavaThread<'exec, 'meta> {
    // state: New -> Runnable
    pub fn start_with_frame(&mut self, frame: impl JavaFrame<'exec, 'meta> + 'exec) {
        assert!(
            self.is_new(),
            "Thread state should be 'New' before getting started."
        );

        self.stack.push(frame);
        self.state = JavaThreadState::Runnable;
    }

    // state: Runnable -> Terminated
    pub fn terminate(&mut self) {
        assert!(
            self.is_runnable(),
            "Thread state should be runnable when being terminated",
        );

        self.state = JavaThreadState::Terminated;
    }
}

impl<'exec, 'meta: 'exec> JavaThread<'exec, 'meta> {
    pub fn push_frame(&mut self, frame: impl JavaFrame<'exec, 'meta> + 'exec) {
        assert!(
            self.is_runnable(),
            "Thread state should be runnable."
        );

        self.stack.push(frame);
    }

    pub fn pop_frame(&mut self) -> Option<Box<dyn JavaFrame<'exec, 'meta> + 'exec>> {
        assert!(
            self.is_runnable() || self.is_terminated(),
            "Thread state should be runnable or terminated",
        );

        let frame = self.stack.pop();
        if self.stack.frames.is_empty() {
            self.state = JavaThreadState::Terminated;
        }
        frame
    }

    pub fn current_frame(&mut self) -> Option<&(dyn JavaFrame<'exec, 'meta> + 'exec)> {
        self.stack.current_frame()
    }

    pub fn current_frame_mut(&mut self) -> Option<&mut (dyn JavaFrame<'exec, 'meta> + 'exec)> {
        self.stack.current_frame_mut()
    }
}
