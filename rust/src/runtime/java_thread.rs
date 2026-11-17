use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JavaThreadId(u64);

impl JavaThreadId {
    pub(crate) fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(
            NEXT_ID
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .expect("Java thread ID exhausted"),
        )
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
    id: JavaThreadId,
    state: JavaThreadState,
    stack: JavaStack<'exec, 'meta>,
}

impl<'exec, 'meta: 'exec> JavaThread<'exec, 'meta> {
    pub fn new() -> Self {
        Self {
            id: JavaThreadId::new(),
            state: JavaThreadState::New,
            stack: JavaStack::new(),
        }
    }
}

impl<'exec, 'meta: 'exec> JavaThread<'exec, 'meta> {
    pub fn id(&self) -> JavaThreadId {
        self.id
    }

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

    fn abort_initialization(&mut self, reason: &str) {
        for frame in &mut self.stack.frames {
            frame.abort(reason);
        }
    }

    pub(crate) fn fail(&mut self, error: &crate::engines::exec_error::ExecError) {
        self.abort_initialization(&format!("{error:?}"));
        self.state = JavaThreadState::Terminated;
    }

    // state: Runnable -> Terminated
    pub fn terminate(&mut self) {
        assert!(
            self.is_runnable(),
            "Thread state should be runnable when being terminated",
        );

        self.abort_initialization("initializing thread terminated");
        self.state = JavaThreadState::Terminated;
    }
}

impl<'exec, 'meta: 'exec> JavaThread<'exec, 'meta> {
    pub fn push_frame(&mut self, frame: impl JavaFrame<'exec, 'meta> + 'exec) {
        assert!(self.is_runnable(), "Thread state should be runnable.");

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
