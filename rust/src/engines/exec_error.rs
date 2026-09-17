use std::marker::PhantomData;

use crate::engines::invocation::Invocation;

#[derive(Debug)]
pub enum ExecErrorKind {
    UnexpectedEOF,
    
    MethodNotFound {
        owner: String,
        name: String,
        desc: String,
    },

    NoCode {
        owner: String,
        name: String,
        desc: String,
    },

    MismatchSlotType,

    StackUnderflow,

    InvalidLocalIndex(usize),

    UnsupportedInstruction,

    NoFrame,
}

#[derive(Debug)]
pub struct ExecError {
    __: PhantomData<()>,
    
    pub invocation: Option<Invocation>,

    pub kind: ExecErrorKind,
}

impl ExecError {
    pub(super) fn new(kind: ExecErrorKind) -> Self {
        Self {
            __: PhantomData,
            invocation: None,
            kind,
        }
    }
    
    pub(super) fn with_invocation(invocation: Invocation, kind: ExecErrorKind) -> Self {
        Self {
            __: PhantomData,
            invocation: Some(invocation),
            kind,
        }
    }
}

pub type ExecResult<T> = Result<T, ExecError>;
