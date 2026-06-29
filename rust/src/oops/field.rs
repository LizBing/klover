use std::cell::OnceCell;

use crate::{class_parser::{attr_info::AttrInfo, field_info::FieldInfo}, oops::{acc_flags::{self, AccFlags}, attr::ConstantValueAttr, cp_entry::{CPEntry, get_utf8}, desc::FieldDesc, resolve_error::{ResolveError, ResolveResult}, symbol_table::SymbolHandle}};

pub struct Field {
    pub acc_flags: AccFlags,
    name: SymbolHandle,
    pub desc: FieldDesc,
    pub offs: Option<usize>,

    constant_value: Option<ConstantValueAttr>
}

impl Field {
    pub fn from(info: &FieldInfo, cp: &[Option<CPEntry>]) -> ResolveResult<Self> {
        let acc_flags = AccFlags::from_bits_truncate(info.acc_flags);
        let name = get_utf8(cp, info.name_idx as usize)?;
        let desc = FieldDesc::from(get_utf8(cp, info.desc_idx as usize)?.utf8())?;
        
        let mut constant_value = None;
        for n in &info.attrs {
            match n {
                AttrInfo::ConstantValue { cp_idx } =>
                    constant_value = Some(ConstantValueAttr::from(*cp_idx as usize, cp)?),

                _ => continue
            }
        }

        Ok(Self {
            acc_flags,
            name,
            desc,
            offs: None,
            constant_value
        })
    }
}
