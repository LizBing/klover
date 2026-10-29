use std::ops::Deref;

use crate::{
    class_loader::oop_storage::OOPStorage,
    class_parser::ClassImage,
    oops::oops_errors::{LinkageError, LinkageResult},
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
    pub interfaces: Box<[SymbolHandle]>,

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
            ifaces.push(SymbolTable::intern(&iface.name));
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
    interfaces: MsBox<'cld, [&'cld NormalKlass<'cld>]>,

    methods: MsBox<'cld, [Method<'cld>]>,
}

impl<'cld> NormalKlass<'cld> {
    pub fn link(
        unlinked: UnlinkedNormalKlass<'cld>,
        super_klass: Option<&'cld NormalKlass<'cld>>,
        interfaces: MsBox<'cld, [&'cld NormalKlass<'cld>]>,
    ) -> LinkageResult<Self> {
        for interface in interfaces.iter() {
            if !interface.is_interface() {
                return Err(LinkageError::NotInterface {
                    name: interface.name().utf8().into(),
                });
            }
        }
        let cld = unlinked.cld;

        Ok(Self {
            name: unlinked.name,
            acc_flags: unlinked.acc_flags,
            cld,
            super_klass,
            interfaces,
            methods: unlinked.methods,
        })
    }
}

impl<'cld> NormalKlass<'cld> {
    pub fn name(&self) -> &SymbolHandle {
        &self.name
    }

    pub(super) fn is_interface(&self) -> bool {
        self.acc_flags.contains(ClassAccessFlags::INTERFACE)
    }

    pub(super) fn super_klass(&self) -> Option<&'cld NormalKlass<'cld>> {
        self.super_klass
    }

    pub(super) fn has_interfaces(&self) -> bool {
        !self.interfaces.is_empty()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::support::{fixture_dir, init_vm, isolated};

    #[test]
    fn links_direct_interfaces_in_declaration_order() {
        isolated(
            concat!(
                module_path!(),
                "::links_direct_interfaces_in_declaration_order"
            ),
            || {
                init_vm(fixture_dir());
                let load =
                    |name| BootstrapCLD::find_normal_klass(&SymbolTable::intern(name)).unwrap();
                let base = load("InterfaceLoading$Base");
                let first = load("InterfaceLoading$First");
                let second = load("InterfaceLoading$Second");
                let root = load("InterfaceLoading$Root");

                assert_eq!(base.interfaces.len(), 2);
                assert!(std::ptr::eq(base.interfaces[0], first));
                assert!(std::ptr::eq(base.interfaces[1], second));
                assert_eq!(first.interfaces.len(), 1);
                assert!(std::ptr::eq(first.interfaces[0], root));
                assert!(!root.has_interfaces());
                assert!(!second.has_interfaces());
                let leaf = load("InterfaceLoading$Leaf");
                assert!(!leaf.has_interfaces());
                assert!(std::ptr::eq(leaf.super_klass().unwrap(), base));
            },
        );
    }

    #[test]
    fn rejects_non_interface_metadata() {
        isolated(
            concat!(module_path!(), "::rejects_non_interface_metadata"),
            || {
                let fixtures = fixture_dir();
                init_vm(fixtures.clone());
                let object =
                    BootstrapCLD::find_normal_klass(&SymbolTable::intern("java/lang/Object"))
                        .unwrap();
                let bytes = std::fs::read(fixtures.join("InterfaceLoading$Base.class")).unwrap();
                let unlinked = UnlinkedNormalKlass::build(
                    crate::class_parser::parse_class(&bytes).unwrap(),
                    None,
                );
                let mut interfaces = MsBox::new_uninit_slice(BootstrapCLD::ms_allocator(), 1);
                interfaces[0].write(object);
                let error =
                    NormalKlass::link(unlinked, Some(object), unsafe { interfaces.assume_init() })
                        .unwrap_err();
                assert!(
                    matches!(error, LinkageError::NotInterface { name } if name == "java/lang/Object")
                );
            },
        );
    }
}
