use crate::class_parser::{attr_info::AttrInfo, class_file::read_attrs, cp_info::ConstantPoolInfo};

use super::{class_reader::ClassReader, parse_error::ParseResult};

/// Parsed field metadata — names resolved to `SymbolHandle`, attributes parsed.
///
/// This is the parsed, ready-to-use representation within a [`ClassFile`].
/// The runtime counterpart (offset, storage layout, etc.) will be a separate
/// `Field` type allocated in metaspace alongside `Klass`.
pub struct FieldInfo {
    pub acc_flags: u16,
    pub name_idx: u16,
    pub desc_idx: u16,
    
    pub attrs: Vec<AttrInfo>,
}
    
impl FieldInfo {
    /// Read and resolve a field from the class file stream.
    pub(crate) fn read(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ParseResult<Self> {
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
