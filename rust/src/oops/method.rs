use std::marker::PhantomData;

pub use cafebabe::MethodAccessFlags;

use crate::{runtime::ms_api::MsAllocator, code::code::Code, oops::{desc::MethodDesc, symbol_table::{SymbolHandle, SymbolTable}}};

#[derive(Debug)]
pub struct Method {
    __: PhantomData<()>,
    
    pub acc_flags: MethodAccessFlags,
    pub name: SymbolHandle,
    pub desc: MethodDesc,

    pub code: Option<Code>,
}

impl Method {
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
            __: PhantomData,
            acc_flags: info.access_flags,
            name,
            desc,
            code,
        }
    }
}
