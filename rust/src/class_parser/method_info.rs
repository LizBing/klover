use crate::class_parser::{acc_flags::AccFlags, attr_info::AttrInfo, parse_error::ParseError};

use super::{class_reader::ClassReader, parse_error::ParseResult};

#[derive(Debug)]
pub struct MethodInfo {
    pub acc_flags: AccFlags,
    pub name_index: u16,
    pub desc_index: u16,
    pub attrs: Vec<AttrInfo>,
}

impl MethodInfo {
    pub fn read(rd: &mut ClassReader) -> ParseResult<Self> {
        let raw_acc_flags = rd.read_u16()?;
        let acc_flags = match AccFlags::from_bits(raw_acc_flags) {
            Some(x) => x,
            None => return Err(ParseError::InvalidAccFlags(raw_acc_flags)),
        };

        let name_index = rd.read_u16()?;
        let desc_index = rd.read_u16()?;
        let attrs_count = rd.read_u16()?;
        let mut attrs = Vec::new();
        for _ in 0..attrs_count {
            attrs.push(AttrInfo::read(rd)?);
        }

        Ok(Self {
            acc_flags,
            name_index,
            desc_index,
            attrs,
        })
    }
}
