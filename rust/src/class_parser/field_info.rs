use super::{class_reader::ClassReader, parse_error::ParseResult};

#[derive(Debug)]
pub struct FieldInfo {}

impl FieldInfo {
    pub fn read(rd: &mut ClassReader) -> ParseResult<Self> {
        unimplemented!()
    }
}
