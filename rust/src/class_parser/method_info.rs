use crate::class_parser::{class_file::read_attrs, cp_info::ConstantPoolInfo};
use crate::class_parser::attr_info::AttrInfo;

use super::{class_reader::ClassReader, parse_error::ParseResult};

/// Parsed method metadata — names resolved to `SymbolHandle`, attributes parsed.
///
/// This is the parsed, ready-to-use representation within a [`ClassFile`].
/// The runtime counterpart (vtable index, entry point, etc.) will be a
/// separate `Method` type allocated in metaspace alongside `Klass`.
pub struct MethodInfo {
    pub acc_flags: u16,
    pub name_idx: u16,
    pub desc_idx: u16,
    pub attrs: Vec<AttrInfo>,
}

impl MethodInfo {
    /// Read and resolve a method from the class file stream.
    pub fn read(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ParseResult<Self> {
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
