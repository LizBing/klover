#[derive(Debug)]
pub enum ParseError {
    EOF,
    InvalidMagic(u32),
    InvalidVersion { minor: u16, major: u16 },
    InvalidCPTag(u8),
    InvalidAccFlags(u16),
    InvalidUtf8(Vec<u8>),
    InvalidCPType,
    InvalidDescriptor { raw: String }
}

pub type ParseResult<T> = Result<T, ParseError>;
