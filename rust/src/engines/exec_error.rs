use std::marker::PhantomData;

use crate::engines::invocation::{self, Invocation};

#[derive(Debug)]
pub enum ExecErrorKind {
    UnexpectedEOF,
    
    MethodNotFound,

    NoCode,

    MismatchSlotType,

    StackUnderflow,

    InvalidLocalIndex(usize),

    UnsupportedInstruction,
}

#[derive(Debug)]
pub struct ExecError {
    __: PhantomData<()>,
    
    pub owner: String,
    pub name: String,
    pub desc: String,

    pub kind: ExecErrorKind,
}

impl ExecError {
    pub(super) fn new(
        owner: &str,
        name: &str,
        desc: &str,
        kind: ExecErrorKind,
    ) -> Self {
        Self {
            __: PhantomData,
            owner: owner.into(),
            name: name.into(),
            desc: desc.into(),
            kind,
        }
    }
    
    pub(super) fn with_invocation(invocation: &Invocation, kind: ExecErrorKind) -> Self {
        Self::new(
            invocation.owner.name.utf8(),
            invocation.method.name.utf8(),
            invocation.method.desc.raw.utf8(),
            kind,
        )
    }
}

pub type ExecResult<T> = Result<T, ExecError>;
