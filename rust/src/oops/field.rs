use std::cell::OnceCell;

use crate::{class_parser::field_info::FieldInfo, oops::{acc_flags::{self, AccFlags}, attr::Attribute, cp_entry::CPEntry, desc::FieldDesc, resolve_error::{ResolveError, ResolveResult}, symbol_table::SymbolHandle}};

pub struct Field {
    acc_flags: AccFlags,
    name: SymbolHandle,
    desc: FieldDesc,
    offs: OnceCell<usize>,
}

impl Field {
    pub fn from(info: &FieldInfo, cp: &[CPEntry]) -> ResolveResult<Self> {
        let acc_flags = AccFlags::from_bits_truncate(info.acc_flags);
        
        let name = match &cp[info.name_idx as usize] {
            CPEntry::Utf8 { handle } => handle.clone(),
            _ => return Err(ResolveError::MismatchCPType)
        };

        let desc = match &cp[info.desc_idx as usize] {
            CPEntry::Utf8 { handle } => {
                FieldDesc::from(handle.utf8())?
            }

            _ => return Err(ResolveError::MismatchCPType)
        };

        Ok(Self {
            acc_flags,
            name,
            desc,
            offs: OnceCell::new(),
        })
    }
}
