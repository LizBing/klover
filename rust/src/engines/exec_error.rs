use crate::{engines::invocation::Invocation, oops::jvalue::JValue};

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

    // Until Java exception dispatch is implemented, integer division by zero
    // exits through the execution error channel.
    DivisionByZero,

    InvalidBranchTarget(usize),

    InvalidLocalIndex(usize),

    UnsupportedInstruction,

    NoFrame,
}

#[derive(Debug)]
pub struct InvocationBrief {
    _private: (),

    pub klass: String,
    pub method: String,
    pub args: Box<[JValue]>,
}

#[derive(Debug)]
pub struct ExecError {
    _private: (),

    pub brief: Option<InvocationBrief>,

    pub kind: ExecErrorKind,
}

impl ExecError {
    pub(super) fn new(kind: ExecErrorKind) -> Self {
        Self {
            _private: (),
            brief: None,
            kind,
        }
    }

    pub(super) fn with_invocation(invocation: Invocation, kind: ExecErrorKind) -> Self {
        let brief = InvocationBrief {
            _private: (),
            klass: invocation.owner().name().utf8().into(),
            method: invocation.method().name().utf8().into(),
            args: invocation.args().into(),
        };
        
        Self {
            _private: (),
            brief: Some(brief),
            kind,
        }
    }
}

pub type ExecResult<T> = Result<T, ExecError>;
