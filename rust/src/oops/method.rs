use crate::{class_loader::ms_box::MSAllocator, class_parser::{attr_info::AttrInfo, method_info::MethodInfo}, oops::{acc_flags::AccFlags, attr::CodeAttr, cp_entry::{CPEntry, get_utf8}, desc::MethodDesc, resolve_error::ResolveResult, symbol_table::SymbolHandle}};

pub struct Method {
    pub acc_flags: AccFlags,
    pub name: SymbolHandle,
    pub desc: MethodDesc,
    pub code: Option<CodeAttr>
}

impl Method {
    pub fn from(info: &MethodInfo, cp: &[Option<CPEntry>], msa: &MSAllocator) -> ResolveResult<Self> {
        let acc_flags = AccFlags::from_bits_truncate(info.acc_flags);
        let name = get_utf8(cp, info.name_idx as usize)?;
        let desc = MethodDesc::from(get_utf8(cp, info.desc_idx as usize)?.utf8())?;

        let mut code = None;
        for n in &info.attrs {
            match n {
                AttrInfo::Code { info } => code = Some(CodeAttr::from(info, cp, msa)?),
                _ => continue
            }
        }

        Ok(Self {
            acc_flags,
            name,
            desc,
            code
        })
    }
}
