use crate::class_parser::MethodAccessFlags;

use crate::{
    class_loader::oop_storage::OopStorage,
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

    acc_flags: MethodAccessFlags,
    pub name: SymbolHandle,
    pub desc: MethodDesc<'klass>,

    pub code: Option<Code<'klass>>,
}

impl<'klass> Method<'klass> {
    pub fn is_public(&self) -> bool {
        self.acc_flags.contains(MethodAccessFlags::PUBLIC)
    }

    pub fn is_private(&self) -> bool {
        self.acc_flags.contains(MethodAccessFlags::PRIVATE)
    }

    pub fn is_protected(&self) -> bool {
        self.acc_flags.contains(MethodAccessFlags::PROTECTED)
    }

    pub fn is_static(&self) -> bool {
        self.acc_flags.contains(MethodAccessFlags::STATIC)
    }

    pub fn is_abstract(&self) -> bool {
        self.acc_flags.contains(MethodAccessFlags::ABSTRACT)
    }

    pub fn is_native(&self) -> bool {
        self.acc_flags.contains(MethodAccessFlags::NATIVE)
    }

    pub fn is_synchronized(&self) -> bool {
        self.acc_flags.contains(MethodAccessFlags::SYNCHRONIZED)
    }

    pub(super) fn build(info: &MethodInfo, msa: &'klass MsAllocator, storage: &OopStorage) -> Self {
        let name = SymbolTable::intern(&info.name);
        let desc = MethodDesc::build(&info.descriptor, msa);

        let code = info
            .attributes
            .code
            .as_ref()
            .map(|cd| Code::build(cd, msa, storage));

        Self {
            _private: (),
            acc_flags: info.access_flags,
            name,
            desc,
            code,
        }
    }
}
