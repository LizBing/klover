use crate::{
    engines::invocation::CallTarget,
    oops::{jvalue::JValue, oops_errors::ResolveError},
};

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

    ArgumentCountMismatch {
        expected: usize,
        actual: usize,
    },

    ArgumentTypeMismatch {
        /// Zero-based index into Invocation::args, including the receiver.
        index: usize,
        expected: String,
        actual: JValue,
    },

    MismatchSlotType,

    StackUnderflow,

    // Until Java exception dispatch is implemented, integer division by zero
    // exits through the execution error channel.
    DivisionByZero,

    InvalidBranchTarget(usize),

    InvalidLocalIndex(usize),

    InsufficientLocalSlots {
        required: usize,
        available: usize,
    },

    UnsupportedInstruction,

    NoFrame,

    Resolve(ResolveError),

    ExpectedStaticMethod {
        owner: String,
        name: String,
        desc: String,
    },

    UnsupportedInvocation {
        owner: String,
        name: String,
        desc: String,
        reason: &'static str,
    },

    UnsupportedClassInitialization {
        owner: String,
    },

    ThreadTerminated,
}

#[derive(Debug)]
pub struct InvocationBrief {
    pub klass: String,
    pub method: String,
    pub desc: String,
    /// Lowered instruction index; absent before instruction execution.
    pub instruction_index: Option<usize>,
}

#[derive(Debug)]
pub struct ExecError {
    pub brief: Option<InvocationBrief>,

    pub kind: ExecErrorKind,
}

impl ExecError {
    pub(super) fn new(kind: ExecErrorKind) -> Self {
        Self { brief: None, kind }
    }

    pub(super) fn with_target(
        target: &CallTarget<'_, '_>,
        instruction_index: Option<usize>,
        kind: ExecErrorKind,
    ) -> Self {
        let brief = InvocationBrief {
            klass: target.declaring_class.name().utf8().into(),
            method: target.method.name.utf8().into(),
            desc: target.method.desc.raw().utf8().into(),
            instruction_index,
        };

        Self {
            brief: Some(brief),
            kind,
        }
    }
}

pub type ExecResult<T> = Result<T, ExecError>;
