use crate::class_parser::{attr_info::{AttrInfo, read_attrs}, cp_info::ConstantPoolInfo};

use super::{class_reader::ClassReader, parse_error::ClassFileResult};

pub struct FieldInfo {
    pub acc_flags: u16,
    pub name_idx: u16,
    pub desc_idx: u16,
    
    pub attrs: Vec<AttrInfo>,
}
    
impl FieldInfo {
    /// Read and resolve a field from the class file stream.
    pub(super) fn read(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ClassFileResult<Self> {
        let acc_flags = rd.read_u16()?;
        
        let name_idx = rd.read_u16()?;
        let desc_idx = rd.read_u16()?;

        let attrs = read_attrs(rd, cp)?;

        Ok(Self {
            acc_flags,
            name_idx,
            desc_idx,
            attrs
        })
    }
}

pub(super) fn read_fields(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ClassFileResult<Vec<FieldInfo>> {
    let fields_count = rd.read_u16()?;
    let mut fields = Vec::with_capacity(fields_count as usize);

    for _ in 0..fields_count {
        fields.push(FieldInfo::read(rd, cp)?);
    }

    Ok(fields)
}
