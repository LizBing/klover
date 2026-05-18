use crate::class_parser::parse_error::{ParseError, ParseResult};

pub struct ClassReader<'a> {
    stream: &'a [u8],
    pos: usize
}

impl<'a> ClassReader<'a> {
    pub fn new(stream: &'a [u8]) -> Self {
        Self {
            stream: stream,
            pos: 0
        }
    }
}

impl ClassReader<'_> {
    pub fn read_u8(&mut self) -> ParseResult<u8> {
        let res = match self.stream.get(self.pos) {
            Some(x) => Ok(*x),
            None => Err(super::parse_error::ParseError::EOF)
        };

        self.pos += 1;
        
        res
    }

    pub fn read_u16(&mut self) -> ParseResult<u16> {
        let low = self.read_u8()?;
        let high = self.read_u8()?;

        Ok(u16::from_be_bytes([low, high]))
    }

    pub fn read_u32(&mut self) -> ParseResult<u32> {
        Ok(u32::from_be_bytes([
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?
        ]))
    }

    pub fn read_i64(&mut self) -> ParseResult<i64> {
        Ok(i64::from_be_bytes([
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?
        ]))
    }
    
    pub fn read_f64(&mut self) -> ParseResult<f64> {
        Ok(f64::from_be_bytes([
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?
        ]))
    }

    pub fn read(&mut self, len: usize) -> ParseResult<&[u8]> {
        let res = match self.stream.get(self.pos..self.pos+len+1) {
            Some(x) => x,
            None => return Err(ParseError::EOF)
        };

        Ok(res)
    }
}


