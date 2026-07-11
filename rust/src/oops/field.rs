use std::cell::OnceCell;

use crate::{
    class_parser::{attr_info::AttrInfo, field_info::FieldInfo},
    oops::{
        acc_flags::AccFlags,
        attr::ConstantValue,
        cp_entry::{CPEntry, get_utf8},
        desc::FieldDesc,
        resolve_error::ResolveResult,
        symbol_table::SymbolHandle,
    },
};

#[derive(Debug)]
pub struct Field {
    pub acc_flags: AccFlags,
    pub name: SymbolHandle,
    pub desc: FieldDesc,
    offs: OnceCell<usize>,

    pub constant_value: Option<ConstantValue>,
}

impl Field {
    pub(super) fn from(info: &FieldInfo, cp: &[OnceCell<CPEntry>]) -> ResolveResult<Self> {
        let acc_flags = AccFlags::from_bits_truncate(info.acc_flags);
        let name = get_utf8(cp, info.name_idx as usize)?;
        let raw_desc = get_utf8(cp, info.desc_idx as usize)?;
        let desc = FieldDesc::from(raw_desc.utf8())?;

        let mut constant_value = None;
        for n in &info.attrs {
            match n {
                AttrInfo::ConstantValue { cp_idx } => {
                    constant_value = Some(ConstantValue::build(*cp_idx as usize, cp)?)
                }

                // ignore other attributes
                _ => continue,
            }
        }

        Ok(Self {
            acc_flags,
            name,
            desc,
            offs: OnceCell::new(),
            constant_value,
        })
    }

    pub(super) fn set_offs(&self, n: usize) {
        self.offs.set(n).unwrap()
    }
}

impl Field {
    pub fn offs(&self) -> usize {
        unsafe { *self.offs.get().unwrap_unchecked() }
    }
}
