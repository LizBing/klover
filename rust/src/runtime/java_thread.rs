use crate::engines::java_frame::JavaFrame;

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

impl JavaThread {}
