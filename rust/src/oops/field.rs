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

pub struct Field {
    pub acc_flags: AccFlags,
    pub name: SymbolHandle,
    pub desc: FieldDesc,
    offs: Option<usize>,

    pub constant_value: Option<ConstantValueAttr>,
}

impl Field {
    pub(super) fn from(info: &FieldInfo, cp: &[Option<CPEntry>]) -> ResolveResult<Self> {
        let acc_flags = AccFlags::from_bits_truncate(info.acc_flags);
        let name = get_utf8(cp, info.name_idx as usize)?;
        let desc = FieldDesc::from(get_utf8(cp, info.desc_idx as usize)?.utf8())?;

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
            offs: None,
            constant_value,
        })
    }

    pub(super) fn set_offs(&mut self, n: usize) {
        self.offs = Some(n)
    }
}

impl Field {
    pub fn offs(&self) -> usize {
        unsafe { self.offs.unwrap_unchecked() }
    }
}
