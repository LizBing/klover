pub use cafebabe::MethodAccessFlags;

use crate::{class_loader::oop_storage::OOPStorage, code::code::Code, oops::{desc::MethodDesc, symbol_table::{SymbolHandle, SymbolTable}}, runtime::ms_api::MsAllocator};

#[derive(Debug)]
pub struct Method<'klass> {
    acc_flags: MethodAccessFlags,
    name: SymbolHandle,
    desc: MethodDesc<'klass>,

    code: Option<Code<'klass>>,
}

impl<'klass> Method<'klass> {
    pub fn name(&self) -> &SymbolHandle {
        &self.name
    }
    pub fn desc(&self) -> &MethodDesc<'klass> {
        &self.desc
    }
    pub fn code(&self) -> Option<&Code<'klass>> {
        self.code.as_ref()
    }

    pub(super) fn build(info: &cafebabe::MethodInfo, msa: &'klass MsAllocator, storage: &OOPStorage) -> Self {
        let name = SymbolTable::intern(&info.name);
        let desc = MethodDesc::build(&info.descriptor, msa);

        let mut code = None;
        for attr in &info.attributes {
            if let cafebabe::attributes::AttributeData::Code(cd) = &attr.data {
                code = Some(Code::build(cd, msa, storage));
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
