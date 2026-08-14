pub enum ExecError {
    EOF,
    InvalidLocalIndex(usize),
    MismatchSlotType,
    NoCode,
    StackUnderflow,
    TooManyArguments,
    UnsupportedInstruction,
}

pub type ExecResult<T> = Result<T, ExecError>;
