use crate::{engine::slot::{Slot, SlotKind}, runtime::runtime_error::StackError};

#[derive(Debug)]
pub enum ExecError {
    Stack(StackError),
    
    NoCurrentFrame,
    MethodHasNoCode,

    UnexpectedEndOfCode {
        bci: usize,
    },

    UnsupportedOpcode {
        opcode: u8,
        bci: usize,
    },

    InvalidBranchTarget {
        from: usize,
        offset: i32,
    },

    InvalidLocalIndex(usize),

    OperandStackOverflow,
    OperandStackUnderflow,

    SlotTypeMismatch {
        expected: SlotKind,
        actual: SlotKind,
    },

    TooManyArguments {
        args: usize,
        max_locals: usize,
    },

    InvalidProgramCounter {
        target: usize,
        code_len: usize,
    }
}

pub type ExecResult<T> = Result<T, ExecError>;

#[derive(Debug)]
pub enum JavaExceptionKind {}
