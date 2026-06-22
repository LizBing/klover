#[derive(Debug)]
pub enum ParseError {
    EOF,
    InvalidMagic(u32),
    InvalidVersion { minor: u16, major: u16 },
    InvalidCPTag(u8),
    InvalidUtf8(Vec<u8>),
    InvalidCPType
}

pub type ParseResult<T> = Result<T, ParseError>;
