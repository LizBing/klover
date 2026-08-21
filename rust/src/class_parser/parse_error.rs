#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub offset: usize,
    pub kind: ParseErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    UnexpectedEof {
        needed: usize,
        remaining: usize,
    },

    InvalidMagic(u32),

    UnsupportedVersion {
        major: u16,
        minor: u16,
    },

    InvalidConstantPoolTag {
        cp_index: u16,
        tag: u8,
    },

    InvalidModifiedUtf8 {
        cp_index: u16,
    },

    InvalidConstantPoolReference {
        index: u16,
        expected: CPKind,
    },

    InvalidAttributeLength {
        name_index: u16,
        declared: usize,
    },

    TrailingBytes {
        remaining: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CPKind {
    Utf8,
    Class,
    NameAndType,
    ConstantValue,
}

pub type ParseResult<T> = Result<T, ParseError>;
