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

    // Until Java exception dispatch is implemented, integer division by zero
    // exits through the execution error channel.
    DivisionByZero,

    InvalidBranchTarget(usize),

    InvalidLocalIndex(usize),

    UnsupportedInstruction,

    NoFrame,
}

#[derive(Debug)]
pub struct ExecError {
    _private: (),

    pub invocation: Option<Invocation>,

    pub kind: ExecErrorKind,
}

impl ExecError {
    pub(super) fn new(kind: ExecErrorKind) -> Self {
        Self {
            _private: (),
            invocation: None,
            kind,
        }
    }

    pub(super) fn with_invocation(invocation: Invocation, kind: ExecErrorKind) -> Self {
        Self {
            _private: (),
            invocation: Some(invocation),
            kind,
        }
    }
}

pub type ExecResult<T> = Result<T, ExecError>;
