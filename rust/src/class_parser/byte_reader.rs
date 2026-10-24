//! Bounds-checked big-endian reader used by the class parser.

use super::error::{ParseError, ParseErrorKind, ParseResult};

pub(crate) struct ByteReader<'src> {
    pos: usize,
    bytes: &'src [u8],
}

impl<'src> ByteReader<'src> {
    pub(crate) fn new(bytes: &'src [u8]) -> Self {
        Self { pos: 0, bytes }
    }

    pub(crate) fn pos(&self) -> usize {
        self.pos
    }

    pub(crate) fn remaining(&self) -> usize {
        self.bytes.len() - self.pos
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.pos == self.bytes.len()
    }

    fn eof(&self) -> ParseError {
        ParseError::new(self.pos, ParseErrorKind::UnexpectedEof)
    }

    pub(crate) fn read_u8(&mut self) -> ParseResult<u8> {
        let byte = *self.bytes.get(self.pos).ok_or_else(|| self.eof())?;
        self.pos += 1;
        Ok(byte)
    }

    pub(crate) fn read_i8(&mut self) -> ParseResult<i8> {
        Ok(self.read_u8()? as i8)
    }

    pub(crate) fn read_u16(&mut self) -> ParseResult<u16> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    pub(crate) fn read_i16(&mut self) -> ParseResult<i16> {
        Ok(self.read_u16()? as i16)
    }

    pub(crate) fn read_u32(&mut self) -> ParseResult<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(crate) fn read_i32(&mut self) -> ParseResult<i32> {
        Ok(self.read_u32()? as i32)
    }

    pub(crate) fn read_u64(&mut self) -> ParseResult<u64> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub(crate) fn read_i64(&mut self) -> ParseResult<i64> {
        Ok(self.read_u64()? as i64)
    }

    pub(crate) fn read_bytes(&mut self, len: usize) -> ParseResult<&'src [u8]> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| ParseError::new(self.pos, ParseErrorKind::UnexpectedEof))?;
        let bytes = self.bytes.get(self.pos..end).ok_or_else(|| self.eof())?;
        self.pos = end;
        Ok(bytes)
    }

    pub(crate) fn skip(&mut self, len: usize) -> ParseResult<()> {
        self.read_bytes(len).map(|_| ())
    }

    pub(crate) fn sub_reader(&mut self, len: usize) -> ParseResult<ByteReader<'src>> {
        Ok(ByteReader::new(self.read_bytes(len)?))
    }
}
