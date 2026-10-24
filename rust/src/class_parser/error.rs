use std::fmt;

#[derive(Debug)]
pub(crate) struct ParseError {
    pub(crate) offset: usize,
    pub(crate) kind: ParseErrorKind,
}

#[derive(Debug)]
pub(crate) enum ParseErrorKind {
    UnexpectedEof,
    BadMagic,
    UnsupportedVersion {
        major: u16,
        minor: u16,
    },
    BadConstantPoolTag(u8),
    BadConstantPoolIndex(u16),
    BadConstantPoolEntry(&'static str),
    InvalidModifiedUtf8,
    /// The text cannot be represented as a Rust `String`.
    /// The original modified-UTF-8 bytes are retained for diagnosis.
    InvalidText {
        bytes: Box<[u8]>,
    },
    InvalidDescriptor(&'static str),
    InvalidName(&'static str),
    DuplicateMember,
    MultipleCodeAttributes,
    InvalidCode(&'static str),
    InvalidInstruction(u8),
    InvalidBranchTarget {
        bci: u32,
        target: i64,
    },
    InvalidSwitch,
    InvalidLocalIndex {
        index: u16,
        max_locals: u16,
    },
    InvalidExceptionTable,
    TrailingBytes,
}

impl ParseError {
    pub(crate) fn new(offset: usize, kind: ParseErrorKind) -> Self {
        Self { offset, kind }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "class-file error at byte {}: {:?}",
            self.offset, self.kind
        )
    }
}

pub(crate) type ParseResult<T> = Result<T, ParseError>;
