use std::sync::Arc;

use crate::tests::support::{TempDir, fixture_dir, init_vm, isolated};
use crate::{
    class_loader::{bs_cld::BootstrapCLD, load_error::LoadErrorKind},
    class_parser::{ClassRef, MethodDescriptor, MethodMemRef, MethodMemRefKind},
    oops::{constant_entries::MethodRef, oops_errors::ResolveError, symbol_table::SymbolTable},
};

fn method_ref(kind: MethodMemRefKind, owner: &str, name: &str, desc: &str) -> MethodRef<'static> {
    MethodRef::build(&MethodMemRef {
        kind,
        owner: Arc::new(ClassRef { name: owner.into() }),
        name: name.into(),
        descriptor: MethodDescriptor::parse(desc).unwrap(),
    })
}

#[test]
fn resolves_declared_inherited_overloaded_and_hidden_methods() {
    isolated(
        concat!(
            module_path!(),
            "::resolves_declared_inherited_overloaded_and_hidden_methods"
        ),
        || {
            init_vm(fixture_dir());
            for (owner, name, desc, declaring_class) in [
                (
                    "MethodResolution$Base",
                    "inherited",
                    "()I",
                    "MethodResolution$Base",
                ),
                (
                    "MethodResolution$Leaf",
                    "inherited",
                    "()I",
                    "MethodResolution$Base",
                ),
                (
                    "MethodResolution$Leaf",
                    "overloaded",
                    "(I)I",
                    "MethodResolution$Base",
                ),
                (
                    "MethodResolution$Leaf",
                    "overloaded",
                    "(J)J",
                    "MethodResolution$Base",
                ),
                (
                    "MethodResolution$Leaf",
                    "hidden",
                    "()I",
                    "MethodResolution$Child",
                ),
            ] {
                let reference = method_ref(MethodMemRefKind::Class, owner, name, desc);
                assert!(reference.resolved.get().is_none());
                let actual = reference.resolve(None).unwrap();
                let expected =
                    BootstrapCLD::find_normal_klass(&SymbolTable::intern(declaring_class))
                        .unwrap()
                        .find_declared_method(name, desc)
                        .unwrap();
                assert!(std::ptr::eq(actual, expected), "{owner}.{name}{desc}");
                assert!(std::ptr::eq(actual, *reference.resolved.get().unwrap()));
                assert!(std::ptr::eq(actual, reference.resolve(None).unwrap()));
            }

            let reference = method_ref(
                MethodMemRefKind::Class,
                "MethodResolution$Leaf",
                "hidden",
                "()I",
            );
            std::thread::scope(|scope| {
                let handles: Vec<_> = (0..4)
                    .map(|_| scope.spawn(|| reference.resolve(None).unwrap()))
                    .collect();
                let expected = reference.resolve(None).unwrap();
                for handle in handles {
                    assert!(std::ptr::eq(handle.join().unwrap(), expected));
                }
            });
        },
    );
}

#[test]
fn reports_load_missing_method_and_reference_kind_errors() {
    isolated(
        concat!(
            module_path!(),
            "::reports_load_missing_method_and_reference_kind_errors"
        ),
        || {
            init_vm(fixture_dir());
            let reference = method_ref(
                MethodMemRefKind::Class,
                "MissingResolutionOwner",
                "test",
                "()V",
            );
            let Err(ResolveError::Load(error)) = reference.resolve(None) else {
                panic!("expected owner load failure");
            };
            assert_eq!(error.klass_name, "MissingResolutionOwner");
            assert!(matches!(error.kind, LoadErrorKind::NotFound));
            assert!(reference.resolved.get().is_none());

            for (name, desc) in [
                ("missing", "()V"),
                ("overloaded", "(I)J"),
                ("overloaded", "()I"),
            ] {
                let reference =
                    method_ref(MethodMemRefKind::Class, "MethodResolution$Leaf", name, desc);
                assert!(matches!(reference.resolve(None),
                    Err(ResolveError::NoSuchMethod { owner, name: actual_name, desc: actual_desc })
                        if owner == "MethodResolution$Leaf" && actual_name == name && actual_desc == desc
                ));
                assert!(reference.resolved.get().is_none());
            }

            for (kind, owner, expected_interface) in [
                (MethodMemRefKind::Class, "MethodResolution$Root", false),
                (MethodMemRefKind::Interface, "MethodResolution$Base", true),
            ] {
                let reference = method_ref(kind, owner, "missing", "()V");
                assert!(matches!(reference.resolve(None),
                    Err(ResolveError::IncompatibleClassChange { owner: actual, expected_interface: expected })
                        if actual == owner && expected == expected_interface
                ));
            }
        },
    );
}

#[test]
fn retries_after_a_failed_owner_load() {
    isolated(
        concat!(module_path!(), "::retries_after_a_failed_owner_load"),
        || {
            let dir = TempDir::new();
            let fixtures = fixture_dir();
            std::fs::create_dir_all(dir.0.join("java/lang")).unwrap();
            std::fs::copy(
                fixtures.join("java/lang/Object.class"),
                dir.0.join("java/lang/Object.class"),
            )
            .unwrap();
            init_vm(dir.0.clone());
            let reference = method_ref(
                MethodMemRefKind::Class,
                "MethodResolution$Base",
                "inherited",
                "()I",
            );
            assert!(matches!(
                reference.resolve(None),
                Err(ResolveError::Load(_))
            ));
            assert!(reference.resolved.get().is_none());

            std::fs::copy(
                fixtures.join("MethodResolution$Base.class"),
                dir.0.join("MethodResolution$Base.class"),
            )
            .unwrap();
            let method = reference.resolve(None).unwrap();
            assert!(std::ptr::eq(method, *reference.resolved.get().unwrap()));
        },
    );
}

#[test]
fn resolves_interface_declarations_and_reports_unsupported_fallback() {
    isolated(
        concat!(
            module_path!(),
            "::resolves_interface_declarations_and_reports_unsupported_fallback"
        ),
        || {
            init_vm(fixture_dir());
            for name in ["declared", "abstractMethod", "defaultMethod"] {
                let reference = method_ref(
                    MethodMemRefKind::Interface,
                    "MethodResolution$Root",
                    name,
                    "()I",
                );
                let actual = reference.resolve(None).unwrap();
                let expected =
                    BootstrapCLD::find_normal_klass(&SymbolTable::intern("MethodResolution$Root"))
                        .unwrap()
                        .find_declared_method(name, "()I")
                        .unwrap();
                assert!(std::ptr::eq(actual, expected));
            }
            for (kind, owner, name) in [
                (
                    MethodMemRefKind::Interface,
                    "MethodResolution$Subinterface",
                    "defaultMethod",
                ),
                (
                    MethodMemRefKind::Interface,
                    "MethodResolution$Subinterface",
                    "declared",
                ),
                (
                    MethodMemRefKind::Interface,
                    "MethodResolution$Root",
                    "missing",
                ),
                (
                    MethodMemRefKind::Class,
                    "MethodResolution$Implementor",
                    "defaultMethod",
                ),
                (
                    MethodMemRefKind::Class,
                    "MethodResolution$Subimplementor",
                    "defaultMethod",
                ),
            ] {
                let reference = method_ref(kind, owner, name, "()I");
                assert!(matches!(reference.resolve(None),
                    Err(ResolveError::UnsupportedInterfaceLookup { owner: actual, name: actual_name, desc })
                        if actual == owner && actual_name == name && desc == "()I"
                ));
            }
        },
    );
}
