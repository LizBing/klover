use super::support::{TempDir, fixture_dir, init_vm, isolated};
use crate::{
    class_loader::{bs_cld::BootstrapCLD, load_error::LoadErrorKind},
    oops::symbol_table::SymbolTable,
};

#[test]
fn loads_superclass_and_reuses_class_identity() {
    isolated(
        concat!(
            module_path!(),
            "::loads_superclass_and_reuses_class_identity"
        ),
        || {
            init_vm(fixture_dir());
            let name = SymbolTable::intern("Cases");
            let first = BootstrapCLD::find_class(&name).unwrap();
            let second = BootstrapCLD::find_class(&name).unwrap();
            assert!(
                std::ptr::eq(first, second),
                "repeated lookup created a new class"
            );
            let klass = first.as_normal_klass().unwrap();
            assert_eq!(klass.name().utf8(), "Cases");
            assert!(klass.find_declared_method("sum", "(I)I").is_some());
            assert!(
                BootstrapCLD::find_normal_klass(&SymbolTable::intern("java/lang/Object")).is_ok()
            );
        },
    );
}

#[test]
fn missing_and_malformed_classes_report_context() {
    isolated(
        concat!(
            module_path!(),
            "::missing_and_malformed_classes_report_context"
        ),
        || {
            let dir = TempDir::new();
            std::fs::write(dir.0.join("Broken.class"), b"not a class file").unwrap();
            init_vm(dir.0.clone());
            let error = BootstrapCLD::find_class(&SymbolTable::intern("Missing")).unwrap_err();
            assert!(matches!(error.kind, LoadErrorKind::NotFound));
            assert_eq!(error.klass_name, "Missing");
            assert!(error.cld_name.is_some());
            let error = BootstrapCLD::find_class(&SymbolTable::intern("Broken")).unwrap_err();
            assert!(matches!(error.kind, LoadErrorKind::Parse(_)));
            assert_eq!(error.klass_name, "Broken");
        },
    );
}

#[test]
fn missing_superclass_does_not_cache_a_partial_class() {
    isolated(
        concat!(
            module_path!(),
            "::missing_superclass_does_not_cache_a_partial_class"
        ),
        || {
            let dir = TempDir::new();
            let fixtures = fixture_dir();
            std::fs::copy(fixtures.join("Cases.class"), dir.0.join("Cases.class")).unwrap();
            init_vm(dir.0.clone());
            let name = SymbolTable::intern("Cases");
            let error = BootstrapCLD::find_class(&name).unwrap_err();
            assert!(matches!(error.kind, LoadErrorKind::NotFound));
            assert_eq!(error.klass_name, "java/lang/Object");
            std::fs::create_dir_all(dir.0.join("java/lang")).unwrap();
            std::fs::copy(
                fixtures.join("java/lang/Object.class"),
                dir.0.join("java/lang/Object.class"),
            )
            .unwrap();
            assert!(
                BootstrapCLD::find_class(&name).is_ok(),
                "failed load poisoned the cache"
            );
        },
    );
}
