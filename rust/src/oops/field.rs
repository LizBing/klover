use std::cell::OnceCell;

use crate::{
    class_parser::{attr_info::AttrInfo, field_info::FieldInfo},
    oops::{
        acc_flags::AccFlags,
        attr::ConstantValueAttr,
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
    /// 原始字段描述符（如 `I` / `Ljava/lang/String;` / `[I`）。
    /// 与 `desc` 同信息，但 intern 后可直接指针比较，供 `find_field` 使用。
    pub raw_desc: SymbolHandle,
    offs: OnceCell<usize>,

    pub constant_value: Option<ConstantValueAttr>,
}

impl Field {
    pub(super) fn from(info: &FieldInfo, cp: &[Option<CPEntry>]) -> ResolveResult<Self> {
        let acc_flags = AccFlags::from_bits_truncate(info.acc_flags);
        let name = get_utf8(cp, info.name_idx as usize)?;
        let raw_desc = get_utf8(cp, info.desc_idx as usize)?;
        let desc = FieldDesc::from(raw_desc.utf8())?;

        let mut constant_value = None;
        for n in &info.attrs {
            match n {
                AttrInfo::ConstantValue { cp_idx } => {
                    constant_value = Some(ConstantValueAttr::from(*cp_idx as usize, cp)?)
                }

                _ => continue,
            }
        }

        Ok(Self {
            acc_flags,
            name,
            desc,
            raw_desc,
            offs: OnceCell::new(),
            constant_value,
        })
    }

    pub(super) fn set_offs(&mut self, n: usize) {
        self.offs.set(n).unwrap()
    }
}

impl Field {
    pub fn offs(&self) -> usize {
        unsafe { *self.offs.get().unwrap_unchecked() }
    }
}
