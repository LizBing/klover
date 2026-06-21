use std::cell::OnceCell;

use crate::class_loader::symbol_handle::SymbolHandle;
use crate::class_parser::attr_info::AttrInfo;
use crate::class_parser::class_file::{read_acc_flags, read_attrs, resolve_symbol};
use crate::class_parser::field_type::FieldType;
use crate::class_parser::acc_flags::AccFlags;

use super::{class_reader::ClassReader, parse_error::ParseResult};

/// Parsed field metadata — names resolved to `SymbolHandle`, attributes parsed.
///
/// This is the parsed, ready-to-use representation within a [`ClassFile`].
/// The runtime counterpart (offset, storage layout, etc.) will be a separate
/// `Field` type allocated in metaspace alongside `Klass`.
pub struct FieldInfo {
    pub acc_flags: AccFlags,
    pub name: SymbolHandle,
    pub desc: FieldType,
    pub offs: OnceCell<usize>,
    pub attrs: Vec<AttrInfo>,
}

impl FieldInfo {
    /// Read and resolve a field from the class file stream.
    pub(crate) fn read(
        rd: &mut ClassReader,
        cp: &[super::cp_info::ConstantPoolInfo]
    ) -> ParseResult<Self> {
        let acc_flags = read_acc_flags(rd)?;
        
        let name_index = rd.read_u16()?;
        let desc_index = rd.read_u16()?;

        let attrs = read_attrs(rd, cp)?;

        let name = resolve_symbol(name_index, cp)?;
        let raw_desc = resolve_symbol(desc_index, cp)?.utf8().clone();
        let desc = FieldType::resolve(raw_desc)?;

        Ok(Self {
            acc_flags: acc_flags,
            name: name,
            desc: desc,
            offs: OnceCell::new(),
            attrs: attrs
        })
    }
}
