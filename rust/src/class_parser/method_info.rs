use crate::class_parser::{class_reader::ClassReader, parse_error::ParseResult};

pub struct MethodInfo {}

impl MethodInfo {
    pub fn read(rd: &mut ClassReader) -> ParseResult<Self> {
        unimplemented!()
    }
}
