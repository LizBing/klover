use crate::engines::interpreter::interpreter_frame::InterpreterFrame;

pub enum JavaFrame {
    Interpreter(InterpreterFrame),
}

pub struct JavaStack {
    frames: Vec<JavaFrame>,
}

impl JavaStack {
    pub fn push(&mut self, f: JavaFrame) {
        self.frames.push(f);
    }

    pub fn pop(&mut self) -> Option<JavaFrame> {
        self.frames.pop()
    }
}
