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
    /// Assemble and validate direct hierarchy metadata. Bytecode verification,
    /// field preparation, and loading constraints are not yet implemented.
    pub fn link(
        unlinked: UnlinkedNormalKlass<'dc>,
        super_klass: Option<&'dc NormalKlass<'dc>>,
        interfaces: MsBox<'dc, [&'dc NormalKlass<'dc>]>,
    ) -> LinkageResult<Self> {
        let name = unlinked.name.utf8();
        match (unlinked.super_name.as_ref(), super_klass) {
            (Some(expected), None) => {
                return Err(LinkageError::SuperNotFound {
                    name: expected.utf8().into(),
                });
            }
            (Some(expected), Some(actual)) if expected != actual.name() => {
                return Err(LinkageError::InvalidSuperclass { name: name.into() });
            }
            (None, Some(_)) => {
                return Err(LinkageError::InvalidSuperclass { name: name.into() });
            }
            _ => {}
        }
        if (name == "java/lang/Object") != super_klass.is_none()
            || (unlinked.acc_flags.contains(ClassAccessFlags::INTERFACE)
                && (name == "java/lang/Object"
                    || super_klass.is_some_and(|klass| klass.name().utf8() != "java/lang/Object")))
        {
            return Err(LinkageError::InvalidSuperclass { name: name.into() });
        }
        if let Some(parent) = super_klass {
            if parent.is_interface() {
                return Err(LinkageError::SuperIsInterface {
                    name: parent.name().utf8().into(),
                });
            }
            if parent.acc_flags.contains(ClassAccessFlags::FINAL) {
                return Err(LinkageError::FinalSuperclass {
                    name: parent.name().utf8().into(),
                });
            }
            if !parent.is_accessible_to(name, unlinked.def_cld) {
                return Err(LinkageError::IllegalAccess {
                    from: name.into(),
                    target: parent.name().utf8().into(),
                });
            }
        }
        for interface in interfaces.iter() {
            if !interface.is_interface() {
                return Err(LinkageError::NotInterface {
                    name: interface.name().utf8().into(),
                });
            }
            if !interface.is_accessible_to(name, unlinked.def_cld) {
                return Err(LinkageError::IllegalAccess {
                    from: name.into(),
                    target: interface.name().utf8().into(),
                });
            }
        }
        if interfaces.len() != unlinked.interfaces.len()
            || interfaces
                .iter()
                .zip(unlinked.interfaces.iter())
                .any(|(actual, expected)| actual.name() != expected)
            || unlinked
                .interfaces
                .iter()
                .enumerate()
                .any(|(index, name)| unlinked.interfaces[..index].contains(name))
        {
            return Err(LinkageError::InvalidInterfaces { name: name.into() });
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

impl NormalKlass<'_> {
    pub fn name(&self) -> &SymbolHandle {
        &self.name
    }

    pub fn is_interface(&self) -> bool {
        self.acc_flags.contains(ClassAccessFlags::INTERFACE)
    }

    /// Check class/interface accessibility from a caller's internal class name
    /// and defining loader. Non-public types require the same runtime package.
    /// The caller may still be unlinked; no caller metadata is retained.
    pub(crate) fn is_accessible_to(
        &self,
        accessing_name: &str,
        accessing_cld: &ClassLoaderData,
    ) -> bool {
        fn package_name(name: &str) -> &str {
            name.rsplit_once('/').map_or("", |(package, _)| package)
        }

        self.acc_flags.contains(ClassAccessFlags::PUBLIC)
            || (std::ptr::eq(self.def_cld, accessing_cld)
                && package_name(self.name.utf8()) == package_name(accessing_name))
    }

    pub(super) fn has_interfaces(&self) -> bool {
        !self.interfaces.is_empty()
    }
}

impl<'cld> NormalKlass<'cld> {
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
    use crate::tests::support::{ensure_metaspace, fixture_dir, hierarchy_bytes};

    fn build<'a>(
        cld: &'a ClassLoaderData,
        name: &str,
        super_klass: Option<&'a NormalKlass<'a>>,
        interfaces: &[&'a NormalKlass<'a>],
    ) -> LinkageResult<&'a NormalKlass<'a>> {
        let bytes = std::fs::read(fixture_dir().join(format!("{name}.class"))).unwrap();
        link_bytes(cld, &bytes, super_klass, interfaces)
            .map(|klass| MsBox::new(cld.ms_allocator(), klass).leak())
    }

    fn link_bytes<'a>(
        cld: &'a ClassLoaderData,
        bytes: &[u8],
        super_klass: Option<&'a NormalKlass<'a>>,
        interfaces: &[&'a NormalKlass<'a>],
    ) -> LinkageResult<NormalKlass<'a>> {
        let image = crate::class_parser::parse_class(bytes).unwrap();
        let unlinked = UnlinkedNormalKlass::build(image, cld);
        let mut slots = MsBox::new_uninit_slice(cld.ms_allocator(), interfaces.len());
        for (slot, interface) in slots.iter_mut().zip(interfaces) {
            slot.write(*interface);
        }
        NormalKlass::link(unlinked, super_klass, unsafe { slots.assume_init() })
    }

    #[test]
    fn accessibility_uses_public_flag_and_runtime_package() {
        ensure_metaspace();
        let defining = ClassLoaderData::new(Some("same name".into()));
        let other = ClassLoaderData::new(Some("same name".into()));
        defining
            .define_klass(&hierarchy_bytes("java/lang/Object", None, &[], 0x21))
            .unwrap();
        for (name, flags) in [("p/Hidden", 0x20), ("p/HiddenIface", 0x600)] {
            let target = defining
                .define_klass(&hierarchy_bytes(name, Some("java/lang/Object"), &[], flags))
                .unwrap();
            let target = target.as_normal().unwrap();
            assert!(target.is_accessible_to("p/Caller", &defining));
            assert!(target.is_accessible_to("p/Outer$Inner", &defining));
            assert!(target.is_accessible_to(name, &defining));
            for caller in ["q/Caller", "p/sub/Caller", "prefix/Caller", "Caller"] {
                assert!(
                    !target.is_accessible_to(caller, &defining),
                    "{name} from {caller}"
                );
            }
            assert!(!target.is_accessible_to("p/Caller", &other));
        }
        for (name, flags) in [("p/Public", 0x21), ("p/PublicIface", 0x601)] {
            let target = defining
                .define_klass(&hierarchy_bytes(name, Some("java/lang/Object"), &[], flags))
                .unwrap();
            let target = target.as_normal().unwrap();
            assert!(target.is_accessible_to("q/Caller", &defining));
            assert!(target.is_accessible_to("q/Caller", &other));
        }
        let default_package = defining
            .define_klass(&hierarchy_bytes(
                "Hidden",
                Some("java/lang/Object"),
                &[],
                0x20,
            ))
            .unwrap();
        let default_package = default_package.as_normal().unwrap();
        assert!(default_package.is_accessible_to("Caller", &defining));
        assert!(!default_package.is_accessible_to("p/Caller", &defining));
        assert!(!default_package.is_accessible_to("Caller", &other));
    }

    #[test]
    fn links_only_accessible_dependencies_across_defining_loaders() {
        ensure_metaspace();
        let defining = ClassLoaderData::new(Some("same name".into()));
        let other = ClassLoaderData::new(Some("same name".into()));
        let object = defining
            .define_klass(&hierarchy_bytes("java/lang/Object", None, &[], 0x21))
            .unwrap();
        let object = object.as_normal().unwrap();
        for (target_name, flags, public, interface) in [
            ("p/HiddenBase", 0x20, false, false),
            ("p/PublicBase", 0x21, true, false),
            ("p/HiddenIface", 0x600, false, true),
            ("p/PublicIface", 0x601, true, true),
        ] {
            let target = defining
                .define_klass(&hierarchy_bytes(
                    target_name,
                    Some("java/lang/Object"),
                    &[],
                    flags,
                ))
                .unwrap();
            let target = target.as_normal().unwrap();
            for (caller_cld, caller_name, same_package) in [
                (&defining, "p/Child", true),
                (&defining, "q/Child", false),
                (&other, "p/Child", false),
                (&other, "q/Child", false),
            ] {
                let interfaces = if interface { vec![target] } else { vec![] };
                let parent = if interface { object } else { target };
                let interface_names = [target_name];
                let bytes = hierarchy_bytes(
                    caller_name,
                    Some(parent.name().utf8()),
                    if interface { &interface_names } else { &[] },
                    0x21,
                );
                let result = link_bytes(caller_cld, &bytes, Some(parent), &interfaces);
                if public || same_package {
                    assert!(result.is_ok(), "{caller_name} -> {target_name}: {result:?}");
                } else {
                    assert!(
                        matches!(result, Err(LinkageError::IllegalAccess { from, target })
                        if from == caller_name && target == target_name)
                    );
                }
            }
        }
    }

    #[test]
    fn links_direct_interfaces_in_declaration_order() {
        ensure_metaspace();
        let cld = ClassLoaderData::new(None);
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
        let cld = ClassLoaderData::new(None);
        let object = build(&cld, "java/lang/Object", None, &[]).unwrap();
        let error = build(&cld, "InterfaceLoading$Base", Some(object), &[object]).unwrap_err();
        assert!(matches!(error, LinkageError::NotInterface { name } if name == "java/lang/Object"));
    }

    #[test]
    fn rejects_missing_and_mismatched_dependencies() {
        ensure_metaspace();
        let cld = ClassLoaderData::new(None);
        let object = build(&cld, "java/lang/Object", None, &[]).unwrap();
        let root = build(&cld, "InterfaceLoading$Root", Some(object), &[]).unwrap();
        let first = build(&cld, "InterfaceLoading$First", Some(object), &[root]).unwrap();
        let second = build(&cld, "InterfaceLoading$Second", Some(object), &[]).unwrap();
        assert!(
            matches!(build(&cld, "InterfaceLoading$Base", None, &[first, second]),
            Err(LinkageError::SuperNotFound { name }) if name == "java/lang/Object")
        );
        assert!(matches!(
            build(&cld, "InterfaceLoading$Leaf", Some(object), &[]),
            Err(LinkageError::InvalidSuperclass { .. })
        ));
        assert!(matches!(
            build(&cld, "InterfaceLoading$Base", Some(object), &[first]),
            Err(LinkageError::InvalidInterfaces { .. })
        ));
        assert!(matches!(
            build(
                &cld,
                "InterfaceLoading$Base",
                Some(object),
                &[second, first]
            ),
            Err(LinkageError::InvalidInterfaces { .. })
        ));
    }
}
