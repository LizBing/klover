use crate::{engine::interpreter::interpreter_frame::InterpreterFrame, runtime::runtime_error::{StackError, StackResult}};

#[derive(Debug)]
pub enum JavaFrame {
    Interpreter(InterpreterFrame),
}

impl JavaFrame {
    fn reserved_slots(&self) -> usize {
        match self {
            Self::Interpreter(x) => x.reserved_slots(),
        }
    }
}

pub struct JavaStack {
    frames: Vec<JavaFrame>,

    used_slots: usize,
    max_slots: usize,
}

impl JavaStack {
    pub fn new(max_slots: usize) -> Self {
        Self {
            frames: Vec::new(),
            used_slots: 0,
            max_slots,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn depth(&self) -> usize {
        self.frames.len()
    }

    pub fn push_interpreter(
        &mut self,
        frame: InterpreterFrame,
    ) -> StackResult<()> {
        let required = frame.reserved_slots();

        let new_used = self
            .used_slots
            .checked_add(required)
            .ok_or(StackError::Overflow)?;

        if new_used > self.max_slots {
            return Err(StackError::Overflow);
        }

        self.used_slots = new_used;
        self.frames.push(JavaFrame::Interpreter(frame));

        Ok(())
    }

    pub fn pop(&mut self) -> Option<JavaFrame> {
        let frame = self.frames.pop()?;
        self.used_slots -= frame.reserved_slots();
        Some(frame)
    }

    pub fn current_interpreter(
        &self,
    ) -> StackResult<&InterpreterFrame> {
        match self.frames.last() {
            Some(JavaFrame::Interpreter(frame)) => Ok(frame),
            None => Err(StackError::Empty),
        }
    }

    pub fn current_interpreter_mut(
        &mut self,
    ) -> StackResult<&mut InterpreterFrame> {
        match self.frames.last_mut() {
            Some(JavaFrame::Interpreter(frame)) => Ok(frame),
            None => Err(StackError::Empty),
        }
    }
}
