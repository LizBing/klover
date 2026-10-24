use std::ops::Deref;

use crate::{
    class_loader::oop_storage::OOPStorage,
    class_parser::ClassImage,
    oops::oops_errors::LinkageResult,
    runtime::ms_api::{MsAllocator, MsBox},
};

pub(crate) use crate::class_parser::ClassAccessFlags;

use crate::{
    class_loader::{bs_cld::BootstrapCLD, class_loader_data::ClassLoaderData},
    oops::{
        method::Method,
        symbol_table::{SymbolHandle, SymbolTable},
    },
};

fn get_msa<'cld>(cld: Option<&'cld ClassLoaderData<'cld>>) -> &'cld MsAllocator {
    match cld {
        Some(cld) => cld.ms_allocator(),
        None => BootstrapCLD::ms_allocator(),
    }
}

fn get_storage<'cld>(cld: Option<&'cld ClassLoaderData<'cld>>) -> &'cld OOPStorage<'cld> {
    match cld {
        Some(cld) => cld.oop_storage(),
        None => BootstrapCLD::oop_storage(),
    }
}

pub struct UnlinkedNormalKlass<'cld> {
    name: SymbolHandle,
    acc_flags: ClassAccessFlags,
    cld: Option<&'cld ClassLoaderData<'cld>>,

    pub super_name: Option<SymbolHandle>,
    interfaces: Box<[SymbolHandle]>,

    methods: MsBox<'cld, [Method<'cld>]>,
}

impl<'cld> UnlinkedNormalKlass<'cld> {
    pub fn build(cf: ClassImage, cld: Option<&'cld ClassLoaderData<'cld>>) -> Self {
        let msa = get_msa(cld);
        let storage = get_storage(cld);

        let name = SymbolTable::intern(cf.this_class_name());
        let super_name = cf.super_class_name().map(|name| SymbolTable::intern(name));

        let mut ifaces = Vec::with_capacity(cf.interfaces().len());
        for iface in cf.interfaces() {
            ifaces.push(SymbolTable::intern(iface));
        }

        let mut methods = MsBox::new_uninit_slice(msa, cf.methods().len());
        for (i, method) in cf.methods().iter().enumerate() {
            methods[i].write(Method::build(method, msa, storage));
        }

        Self {
            name,
            acc_flags: cf.access_flags(),
            cld,
            super_name,
            interfaces: ifaces.into(),
            methods: unsafe { methods.assume_init() },
        }
    }
}

#[derive(Debug)]
pub struct NormalKlass<'cld> {
    name: SymbolHandle,
    acc_flags: ClassAccessFlags,
    cld: Option<&'cld ClassLoaderData<'cld>>,

    super_klass: Option<&'cld NormalKlass<'cld>>,

    methods: MsBox<'cld, [Method<'cld>]>,
}

impl<'cld> NormalKlass<'cld> {
    pub fn link(
        unlinked: UnlinkedNormalKlass<'cld>,
        super_klass: Option<&'cld NormalKlass<'cld>>,
    ) -> LinkageResult<Self> {
        let cld = unlinked.cld;

        Ok(Self {
            name: unlinked.name,
            acc_flags: unlinked.acc_flags,
            cld,
            super_klass,
            methods: unlinked.methods,
        })
    }
}

impl<'cld> NormalKlass<'cld> {
    pub fn name(&self) -> &SymbolHandle {
        &self.name
    }

    pub fn find_declared_method(&self, name: &str, desc: &str) -> Option<&Method<'cld>> {
        for method in self.methods.deref() {
            if method.name.utf8() == name && method.desc.raw().utf8() == desc {
                return Some(method);
            }
        }

        None
    }
}
