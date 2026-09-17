use crate::runtime::ms_api::{MsAllocator, MsBox, MsRef};
use std::{ops::Deref, ptr::NonNull};

pub use cafebabe::ClassAccessFlags;

use crate::{class_loader::{bs_cld::BootstrapCLD, class_loader_data::ClassLoaderData}, oops::{fields::Fields, method::Method, oops_errors::LinkageError, symbol_table::{SymbolHandle, SymbolTable}}};

fn get_msa(cld: Option<&ClassLoaderData>) -> &MsAllocator {
    match cld {
        Some(cld) => cld.ms_allocator(),
        None => BootstrapCLD::ms_allocator(),
    }
}

pub struct UnlinkedNormalKlass<'cld> {
    name: SymbolHandle,
    acc_flags: ClassAccessFlags,
    cld: Option<&'cld ClassLoaderData>,
    
    super_name: Option<SymbolHandle>,
    interfaces: Box<[SymbolHandle]>,

    methods: MsBox<[Method]>,
}

impl<'cld> UnlinkedNormalKlass<'cld> {
    pub fn build(cf: cafebabe::ClassFile, cld: Option<&'cld ClassLoaderData>) -> Self {
        let msa = get_msa(cld);
        
        let name = SymbolTable::intern(&cf.this_class);
        let super_name = cf.super_class.map(|n| SymbolTable::intern(&n));

        let mut ifaces = Vec::with_capacity(cf.interfaces.len());
        for iface in cf.interfaces {
            ifaces.push(SymbolTable::intern(&iface));
        }

        let methods = msa.calloc(cf.methods.len());
        for (i, v) in cf.methods.iter().enumerate() {
            methods[i].write(Method::build(v, msa));
        }

        unsafe {
            Self {
                name,
                acc_flags: cf.access_flags,
                cld,
                super_name,
                interfaces: ifaces.into(),
                methods: MsBox::from_raw(methods.assume_init_mut()),
            }
        }
    }
}

#[derive(Debug)]
pub struct NormalKlass {
    name: SymbolHandle,
    acc_flags: ClassAccessFlags,
    cld: Option<NonNull<ClassLoaderData>>,

    super_klass: Option<MsRef<NormalKlass>>,

    methods: MsBox<[Method]>,
}

impl TryFrom<UnlinkedNormalKlass<'_>> for NormalKlass {
    type Error = LinkageError;

    fn try_from(value: UnlinkedNormalKlass<'_>) -> Result<Self, Self::Error> {        
        let cld = value.cld.map(|cld| cld.into());

        let super_klass = match value.super_name {
            None => None,

            Some(skn) => {
                let sk = match value.cld {
                    None => BootstrapCLD::find_class(skn.utf8()),
                    Some(cld) => cld.load_class(skn.utf8()),
                }.map_err(|_| LinkageError::SuperNotFound { name: skn.utf8().into() })?;

                match sk.as_normal_klass_ref() {
                    None => return Err(LinkageError::NotNormalKlass { name: skn.utf8().into() }),
                    Some(x) => Some(x),
                }
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

impl NormalKlass {
    pub fn name(&self) -> &SymbolHandle {
        &self.name
    }

    pub fn find_declared_method(&self, name: &str, desc: &str) -> Option<MsRef<Method>> {
        for method in self.methods.deref() {
            if method.name().utf8() == name && method.desc().raw().utf8() == desc {
                unsafe { return Some(MsRef::from_raw(method.into())); }
            }
        }

        None
    }
}
