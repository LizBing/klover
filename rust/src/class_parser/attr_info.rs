use crate::class_parser::{cp_info::ConstantPoolInfo, parse_error::ParseResult};

pub struct AttrInfo {}

impl AttrInfo {
    pub fn read(name_index: u16, data: &[u8], cp: &[ConstantPoolInfo]) -> ParseResult<Self> {
        unimplemented!()
    }
}
