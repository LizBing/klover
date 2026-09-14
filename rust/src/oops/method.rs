use std::marker::PhantomData;

pub use cafebabe::MethodAccessFlags;

use crate::{class_loader::ms_api::MSAllocator, code::code::Code, oops::{desc::MethodDesc, symbol_table::{SymbolHandle, SymbolTable}}};

pub struct Method {
    __: PhantomData<()>,
    
    pub acc_flags: MethodAccessFlags,
    pub name: SymbolHandle,
    pub desc: MethodDesc,

    pub code: Option<Code>,
}

impl Method {
    pub(super) fn build(info: &cafebabe::MethodInfo, msa: &MSAllocator) -> Self {
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
