use std::{ops::Deref, ptr::NonNull};

pub use cafebabe::ClassAccessFlags;

use crate::{class_loader::{bs_cld::BootstrapCLD, class_loader_data::ClassLoaderData, ms_api::{MSAllocator, MSBox, MSRef}}, oops::{fields::Fields, method::Method, oops_errors::LinkageError, symbol_table::{SymbolHandle, SymbolTable}}};

fn get_msa(cld: Option<&ClassLoaderData>) -> &MSAllocator {
    match cld {
        Some(cld) => &cld.msa,
        None => BootstrapCLD::ms_allocator(),
    }
}

pub struct UnlinkedNormalKlass<'cld> {
    name: SymbolHandle,
    acc_flags: ClassAccessFlags,
    cld: Option<&'cld ClassLoaderData>,
    
    super_name: Option<SymbolHandle>,
    interfaces: Box<[SymbolHandle]>,

    methods: MSBox<[Method]>,
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
                methods: MSBox::from_raw(methods.assume_init_mut()),
            }
        }
    }
}

pub struct NormalKlass {
    pub name: SymbolHandle,
    pub acc_flags: ClassAccessFlags,
    cld: Option<NonNull<ClassLoaderData>>,

    super_klass: Option<MSRef<NormalKlass>>,

    methods: MSBox<[Method]>,
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
    pub fn find_declared_method(&self, name: &str, desc: &str) -> Option<MSRef<Method>> {
        for method in self.methods.deref() {
            if method.name.utf8() == name && method.desc.raw.utf8() == desc {
                unsafe { return Some(MSRef::from_raw(method.into())); }
            }
        }

        None
    }
}
