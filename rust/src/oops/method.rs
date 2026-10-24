pub(crate) use crate::class_parser::MethodAccessFlags;

use crate::{
    class_loader::oop_storage::OOPStorage,
    class_parser::MethodInfo,
    code::code::Code,
    oops::{
        desc::MethodDesc,
        symbol_table::{SymbolHandle, SymbolTable},
    },
    runtime::ms_api::MsAllocator,
};

#[derive(Debug)]
pub struct Method<'klass> {
    _private: (),

    pub acc_flags: MethodAccessFlags,
    pub name: SymbolHandle,
    pub desc: MethodDesc<'klass>,

    pub code: Option<Code<'klass>>,
}

impl<'klass> Method<'klass> {
    pub(super) fn build(info: &MethodInfo, msa: &'klass MsAllocator, storage: &OOPStorage) -> Self {
        let name = SymbolTable::intern(&info.name);
        let desc = MethodDesc::build(&info.descriptor, msa);

        let code = info.code.as_ref().map(|cd| Code::build(cd, msa, storage));

        Self {
            _private: (),
            acc_flags: info.access_flags,
            name,
            desc,
            code,
        }
    }
}
