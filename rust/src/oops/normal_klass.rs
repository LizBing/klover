use crate::{class_loader::oop_storage::OOPStorage, runtime::ms_api::{MsAllocator, MsBox}};
use std::ops::Deref;

pub use cafebabe::ClassAccessFlags;

use crate::{class_loader::{bs_cld::BootstrapCLD, class_loader_data::ClassLoaderData}, oops::{method::Method, oops_errors::LinkageError, symbol_table::{SymbolHandle, SymbolTable}}};

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

    super_name: Option<SymbolHandle>,
    interfaces: Box<[SymbolHandle]>,

    methods: MsBox<'cld, [Method<'cld>]>,
}

impl<'cld> UnlinkedNormalKlass<'cld> {
    pub fn build(cf: cafebabe::ClassFile, cld: Option<&'cld ClassLoaderData<'cld>>) -> Self {
        let msa = get_msa(cld);
        let storage = get_storage(cld);

        let name = SymbolTable::intern(&cf.this_class);
        let super_name = cf.super_class.map(|n| SymbolTable::intern(&n));

        let mut ifaces = Vec::with_capacity(cf.interfaces.len());
        for iface in cf.interfaces {
            ifaces.push(SymbolTable::intern(&iface));
        }

        let mut methods = MsBox::new_uninit_slice(msa, cf.methods.len());
        for (i, v) in cf.methods.iter().enumerate() {
            methods[i].write(Method::build(v, msa, storage));
        }

        Self {
            name,
            acc_flags: cf.access_flags,
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

impl<'cld> TryFrom<UnlinkedNormalKlass<'cld>> for NormalKlass<'cld> {
    type Error = LinkageError;

    fn try_from(value: UnlinkedNormalKlass<'cld>) -> Result<Self, Self::Error> {
        let cld = value.cld;

        let super_klass = match value.super_name {
            None => None,

            Some(skn) => {
                let sk = match value.cld {
                    None => BootstrapCLD::find_normal_klass(&skn),
                    Some(cld) => cld.load_class(skn.utf8()),
                }.map_err(|_| LinkageError::SuperNotFound { name: skn.utf8().into() })?;

                Some(sk)
            }
        };

        Ok(Self {
            name: value.name,
            acc_flags: value.acc_flags,
            cld,
            super_klass,
            methods: value.methods,
        })
    }
}

impl<'cld> NormalKlass<'cld> {
    pub fn name(&self) -> &SymbolHandle {
        &self.name
    }

    pub fn find_declared_method(&self, name: &str, desc: &str) -> Option<&Method<'cld>> {
        for method in self.methods.deref() {
            if method.name().utf8() == name && method.desc().raw().utf8() == desc {
                return Some(method);
            }
        }

        None
    }
}
