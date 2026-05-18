use crate::class_parser::{class_reader::ClassReader, parse_error::ParseResult};

pub struct FieldInfo {}

impl FieldInfo {
    pub fn read(rd: &mut ClassReader) -> ParseResult<Self> {
        unimplemented!()
    }
}
