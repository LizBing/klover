pub use cafebabe::MethodAccessFlags;

use crate::{runtime::ms_api::MsAllocator, code::code::Code, oops::{desc::MethodDesc, symbol_table::{SymbolHandle, SymbolTable}}};

#[derive(Debug)]
pub struct Method {
    acc_flags: MethodAccessFlags,
    name: SymbolHandle,
    desc: MethodDesc,

    code: Option<Code>,
}

impl Method {
    pub fn name(&self) -> &SymbolHandle {
        &self.name
    }
    pub fn desc(&self) -> &MethodDesc {
        &self.desc
    }
    pub fn code(&self) -> Option<&Code> {
        self.code.as_ref()
    }

    pub(super) fn build(info: &cafebabe::MethodInfo, msa: &MsAllocator) -> Self {
        let name = SymbolTable::intern(&info.name);
        let desc = MethodDesc::build(&info.descriptor, msa);

        let mut code = None;
        for attr in &info.attributes {
            if let cafebabe::attributes::AttributeData::Code(cd) = &attr.data {
                code = Some(Code::build(cd, msa));
            }
        }

        Self {
            acc_flags: info.access_flags,
            name,
            desc,
            code,
        }
    }
}
