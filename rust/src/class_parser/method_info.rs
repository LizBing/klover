use crate::class_parser::{attr_info::read_attrs, cp_info::ConstantPoolInfo};
use crate::class_parser::attr_info::AttrInfo;

use super::{class_reader::ClassReader, parse_error::ParseResult};

pub struct MethodInfo {
    pub acc_flags: u16,
    pub name_idx: u16,
    pub desc_idx: u16,
    pub attrs: Vec<AttrInfo>,
}

impl MethodInfo {
    /// Read and resolve a method from the class file stream.
    pub(super) fn read(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ParseResult<Self> {
        let acc_flags = rd.read_u16()?;

        let name_idx= rd.read_u16()?;
        let desc_idx = rd.read_u16()?;

        let attrs = read_attrs(rd, cp)?;

        Ok(Self {
            acc_flags,
            name_idx,
            desc_idx,
            attrs,
        })
    }
}

pub(super) fn read_methods(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ParseResult<Vec<MethodInfo>> {
    let methods_count = rd.read_u16()?;
    let mut methods = Vec::with_capacity(methods_count as usize);
    for _ in 0..methods_count {
        methods.push(MethodInfo::read(rd, cp)?);
    }

    Ok(methods)
}
