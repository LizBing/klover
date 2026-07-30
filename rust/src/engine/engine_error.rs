use crate::{engine::slot::SlotKind, runtime::runtime_error::StackError};

impl From<StackError> for ExecError {
    fn from(value: StackError) -> Self {
        Self::Stack(value)
    }
}

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

    InvalidConstantPoolIndex(usize),
    InvalidLdcConstant {
        index: usize,
    },
    UnsupportedLdcConstant {
        index: usize,
    },

    OperandStackOverflow,
    OperandStackUnderflow,
    InvalidOperandStackShape,
    InvalidStackOperation {
        opcode: u8,
    },

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaExceptionKind {
    ArithmeticException,
}
