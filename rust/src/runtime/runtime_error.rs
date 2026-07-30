#[derive(Debug)]
pub enum ThreadError {
    InvalidID,
    IDExhausted,
    AlreadyStarted,
}

pub type ThreadResult<T> = Result<T, ThreadError>;

#[derive(Debug)]
pub enum StackError {
    Overflow,
    Empty,
}


pub type StackResult<T> = Result<T, StackError>;
