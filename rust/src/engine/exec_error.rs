use crate::{
    engine::slot::SlotKind,
    oops::oops_errors::{ClassInitError, ResolveError},
    runtime::runtime_error::StackError,
};

impl From<StackError> for ExecError {
    fn from(value: StackError) -> Self {
        Self::Stack(value)
    }
}

impl From<ResolveError> for ExecError {
    fn from(value: ResolveError) -> Self {
        Self::Resolve(value)
    }
}

impl From<ClassInitError> for ExecError {
    fn from(value: ClassInitError) -> Self {
        Self::ClassInitialization(value)
    }
}

#[derive(Debug)]
pub enum ExecError {
    Stack(StackError),
    Resolve(ResolveError),
    ClassInitialization(ClassInitError),
    InvalidClassInitializerReturn,
    InvalidClassInitializationFrameState,
    IncompatibleStaticCall,
    IncompatibleStaticFieldAccess,
    InvalidStaticFieldStorage,
    InvalidFieldValue,
    InvalidConstantValue,
    UnsupportedStringConstantValue,

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
    },
}

pub type ExecResult<T> = Result<T, ExecError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaExceptionKind {
    ArithmeticException,
    NoClassDefFoundError,
}
