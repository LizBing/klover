use crate::class_parser::{class_reader::ClassReader, parse_error::ParseResult};

pub enum AttrInfo {}

impl AttrInfo {
    pub fn read(rd: &mut ClassReader) -> ParseResult<Self> {
        unimplemented!()
    }
}
