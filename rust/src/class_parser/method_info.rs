use super::{class_reader::ClassReader, parse_error::ParseResult};

#[derive(Debug)]
pub struct MethodInfo {}

impl MethodInfo {
    pub fn read(rd: &mut ClassReader) -> ParseResult<Self> {
        unimplemented!()
    }
}
