use crate::class_parser::parse_error::ClassFileErrorKind;

use super::parse_error::{ClassFileError, ClassFileResult};

pub struct ClassReader<'a> {
    stream: &'a [u8],
    pos: usize,
}

impl<'a> ClassReader<'a> {
    pub fn new(stream: &'a [u8]) -> Self {
        Self {
            stream: stream,
            pos: 0,
        }
    }
}

impl ClassReader<'_> {
    pub fn read_u8(&mut self) -> ClassFileResult<u8> {
        let s = self.read(size_of::<u8>())?;
        Ok(s[0])
    }


    pub fn read_u16(&mut self) -> ClassFileResult<u16> {
        let s = self.read(size_of::<u16>())?;
        Ok(u16::from_be_bytes(s.try_into().unwrap()))
    }

    pub fn read_u32(&mut self) -> ClassFileResult<u32> {
        let s = self.read(size_of::<u32>())?;
        Ok(u32::from_be_bytes(s.try_into().unwrap()))
    }

    pub fn read_i32(&mut self) -> ClassFileResult<i32> {
        let s = self.read(size_of::<i32>())?;
        Ok(i32::from_be_bytes(s.try_into().unwrap()))
    }

    pub fn read_f32(&mut self) -> ClassFileResult<f32> {
        let s = self.read(size_of::<f32>())?;
        Ok(f32::from_be_bytes(s.try_into().unwrap()))
    }

    pub fn read_i64(&mut self) -> ClassFileResult<i64> {
        let s = self.read(size_of::<i64>())?;
        Ok(i64::from_be_bytes(s.try_into().unwrap()))
    }

    pub fn read_f64(&mut self) -> ClassFileResult<f64> {
        let s = self.read(size_of::<f64>())?;
        Ok(f64::from_be_bytes(s.try_into().unwrap()))
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.stream.len() - self.pos
    }

    pub fn read(&mut self, len: usize) -> ClassFileResult<&[u8]> {
        let offset = self.pos;

        let end = self.pos.checked_add(len).ok_or(ClassFileError {
            offset,
            kind: ClassFileErrorKind::UnexpectedEof {
                needed: len,
                remaining: self.remaining(),
            },
        })?;

        let bytes = self.stream.get(self.pos..end).ok_or(ClassFileError {
            offset,
            kind: ClassFileErrorKind::UnexpectedEof {
                needed: len,
                remaining: self.remaining(),
            },
        })?;

        self.pos = end;
        
        Ok(bytes)
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }
}
