use std::ops::Deref;

use crate::{
    class_parser::ClassImage,
    oops::oops_errors::{LinkageError, LinkageResult},
    runtime::ms_api::MsBox,
};

pub(crate) use crate::class_parser::ClassAccessFlags;

use crate::{
    class_loader::class_loader_data::ClassLoaderData,
    oops::{
        method::Method,
        symbol_table::{SymbolHandle, SymbolTable},
    },
};

pub struct UnlinkedNormalKlass<'dc> {
    name: SymbolHandle,
    acc_flags: ClassAccessFlags,
    def_cld: &'dc ClassLoaderData,

    pub super_name: Option<SymbolHandle>,
    pub interfaces: Box<[SymbolHandle]>,

    methods: MsBox<'dc, [Method<'dc>]>,
}

impl<'dc> UnlinkedNormalKlass<'dc> {
    pub fn build(cf: ClassImage, cld: &'dc ClassLoaderData) -> Self {
        let msa = cld.ms_allocator();
        let storage = cld.oop_storage();
        
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
            def_cld: cld,
            super_name,
            interfaces: ifaces.into(),
            methods: unsafe { methods.assume_init() },
        }
    }
}

#[derive(Debug)]
pub struct NormalKlass<'dc> {
    name: SymbolHandle,
    acc_flags: ClassAccessFlags,
    pub(super) def_cld: &'dc ClassLoaderData,

    pub(super) super_klass: Option<&'dc NormalKlass<'dc>>,
    interfaces: MsBox<'dc, [&'dc NormalKlass<'dc>]>,

    methods: MsBox<'dc, [Method<'dc>]>,
}

impl<'dc> NormalKlass<'dc> {
    pub fn link(
        unlinked: UnlinkedNormalKlass<'dc>,
        super_klass: Option<&'dc NormalKlass<'dc>>,
        interfaces: MsBox<'dc, [&'dc NormalKlass<'dc>]>,
    ) -> LinkageResult<Self> {
        for interface in interfaces.iter() {
            if !interface.is_interface() {
                return Err(LinkageError::NotInterface {
                    name: interface.name().utf8().into(),
                });
            }
        }

        Ok(Self {
            name: unlinked.name,
            acc_flags: unlinked.acc_flags,
            def_cld: unlinked.def_cld,
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
    use crate::tests::support::{ensure_metaspace, fixture_dir};

    fn build<'a>(
        cld: &'a ClassLoaderData,
        name: &str,
        super_klass: Option<&'a NormalKlass<'a>>,
        interfaces: &[&'a NormalKlass<'a>],
    ) -> LinkageResult<&'a NormalKlass<'a>> {
        let bytes = std::fs::read(fixture_dir().join(format!("{name}.class"))).unwrap();
        let image = crate::class_parser::parse_class(&bytes).unwrap();
        let unlinked = UnlinkedNormalKlass::build(image, cld);
        let mut slots = MsBox::new_uninit_slice(cld.ms_allocator(), interfaces.len());
        for (slot, interface) in slots.iter_mut().zip(interfaces) {
            slot.write(*interface);
        }
        NormalKlass::link(unlinked, super_klass, unsafe { slots.assume_init() })
            .map(|klass| MsBox::new(cld.ms_allocator(), klass).leak())
    }

    #[test]
    fn links_direct_interfaces_in_declaration_order() {
        ensure_metaspace();
        let cld = ClassLoaderData::new();
        let object = build(&cld, "java/lang/Object", None, &[]).unwrap();
        let root = build(&cld, "InterfaceLoading$Root", Some(object), &[]).unwrap();
        let first = build(&cld, "InterfaceLoading$First", Some(object), &[root]).unwrap();
        let second = build(&cld, "InterfaceLoading$Second", Some(object), &[]).unwrap();
        let base = build(
            &cld,
            "InterfaceLoading$Base",
            Some(object),
            &[first, second],
        )
        .unwrap();

        assert_eq!(base.interfaces.len(), 2);
        assert!(std::ptr::eq(base.interfaces[0], first));
        assert!(std::ptr::eq(base.interfaces[1], second));
        assert_eq!(first.interfaces.len(), 1);
        assert!(std::ptr::eq(first.interfaces[0], root));
        assert!(!root.has_interfaces());
        assert!(!second.has_interfaces());
        let leaf = build(&cld, "InterfaceLoading$Leaf", Some(base), &[]).unwrap();
        assert!(!leaf.has_interfaces());
        assert!(std::ptr::eq(leaf.super_klass.unwrap(), base));
    }

    #[test]
    fn rejects_non_interface_metadata() {
        ensure_metaspace();
        let cld = ClassLoaderData::new();
        let object = build(&cld, "java/lang/Object", None, &[]).unwrap();
        let error = build(&cld, "InterfaceLoading$Base", Some(object), &[object]).unwrap_err();
        assert!(matches!(error, LinkageError::NotInterface { name } if name == "java/lang/Object"));
    }
}
