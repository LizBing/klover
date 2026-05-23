use super::{class_reader::ClassReader, parse_error::ParseResult};

#[derive(Debug)]
pub enum AttrInfo {}

impl AttrInfo {
    pub fn read(rd: &mut ClassReader) -> ParseResult<Self> {
        unimplemented!()
    }
}
