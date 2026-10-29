use super::support::{TempDir, fixture_dir, init_vm, isolated};
use crate::{
    class_loader::{bs_cld::BootstrapCLD, load_error::LoadErrorKind},
    oops::symbol_table::SymbolTable,
};

#[test]
fn missing_interface_does_not_cache_a_partial_class() {
    isolated(
        concat!(
            module_path!(),
            "::missing_interface_does_not_cache_a_partial_class"
        ),
        || {
            let dir = TempDir::new();
            let fixtures = fixture_dir();
            std::fs::create_dir_all(dir.0.join("java/lang")).unwrap();
            for file in [
                "java/lang/Object.class",
                "InterfaceLoading$Base.class",
                "InterfaceLoading$First.class",
                "InterfaceLoading$Root.class",
            ] {
                std::fs::copy(fixtures.join(file), dir.0.join(file)).unwrap();
            }
            init_vm(dir.0.clone());
            let name = SymbolTable::intern("InterfaceLoading$Base");
            let error = BootstrapCLD::find_klass(&name).unwrap_err();
            assert!(matches!(error.kind, LoadErrorKind::NotFound));
            assert_eq!(error.klass_name, "InterfaceLoading$Second");
            std::fs::copy(
                fixtures.join("InterfaceLoading$Second.class"),
                dir.0.join("InterfaceLoading$Second.class"),
            )
            .unwrap();
            assert!(
                BootstrapCLD::find_klass(&name).is_ok(),
                "failed interface load poisoned the cache"
            );
        },
    );
}

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
            let first = BootstrapCLD::find_klass(&name).unwrap();
            let second = BootstrapCLD::find_klass(&name).unwrap();
            assert!(
                std::ptr::eq(first, second),
                "repeated lookup created a new class"
            );
            let klass = first.as_normal().unwrap();
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
            let error = BootstrapCLD::find_klass(&SymbolTable::intern("Missing")).unwrap_err();
            assert!(matches!(error.kind, LoadErrorKind::NotFound));
            assert_eq!(error.klass_name, "Missing");
            assert!(error.cld_name.is_some());
            let error = BootstrapCLD::find_klass(&SymbolTable::intern("Broken")).unwrap_err();
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
            let error = BootstrapCLD::find_klass(&name).unwrap_err();
            assert!(matches!(error.kind, LoadErrorKind::NotFound));
            assert_eq!(error.klass_name, "java/lang/Object");
            std::fs::create_dir_all(dir.0.join("java/lang")).unwrap();
            std::fs::copy(
                fixtures.join("java/lang/Object.class"),
                dir.0.join("java/lang/Object.class"),
            )
            .unwrap();
            assert!(
                BootstrapCLD::find_klass(&name).is_ok(),
                "failed load poisoned the cache"
            );
        },
    );
}

#[test]
fn parses_javac_attributes_and_bootstrap_dependencies() {
    use crate::class_parser::{Opcode, parse_class};
    let fixtures = fixture_dir();
    let read = |name: &str| {
        let bytes = std::fs::read(fixtures.join(name)).unwrap();
        parse_class(&bytes).unwrap_or_else(|error| panic!("{name}: {error}"))
    };
    let image = read("ParserAttributes.class");
    let attrs = image.attributes();
    assert!(attrs.signature.is_some());
    assert!(attrs.deprecated);
    assert!(attrs.inner_classes.is_some());
    assert!(attrs.annotations.visible.is_some());
    assert!(attrs.annotations.invisible.is_some());
    assert!(attrs.annotations.visible_types.is_some());
    let bootstrap = &attrs.bootstrap_methods.as_ref().unwrap()[0];
    let crate::class_parser::MethodHandleTarget::Method(bootstrap_target) =
        &bootstrap.method.target
    else {
        panic!("expected bootstrap method target")
    };
    assert_eq!(bootstrap_target.name, "metafactory");
    assert_eq!(bootstrap.arguments.len(), 3);
    assert!(
        image
            .fields()
            .iter()
            .filter(|field| field.name == "NUMBER" || field.name == "TEXT")
            .all(|field| field.attributes.constant_value.is_some())
    );
    let field = image
        .fields()
        .iter()
        .find(|field| field.name == "field")
        .unwrap();
    assert!(field.attributes.signature.is_some());
    assert!(field.attributes.annotations.visible_types.is_some());
    let run = image
        .methods()
        .iter()
        .find(|method| method.name == "run")
        .unwrap();
    assert!(run.attributes.exceptions.is_some());
    assert!(run.attributes.visible_parameter_annotations.is_some());
    assert!(run.attributes.invisible_parameter_annotations.is_some());
    let code = run.attributes.code.as_ref().unwrap();
    assert!(code.attributes.stack_map_table.is_some());
    assert!(code.attributes.visible_type_annotations.is_some());
    assert!(!code.exception_table.is_empty());
    let lambda = image
        .methods()
        .iter()
        .find(|method| method.name == "lambda")
        .unwrap();
    let dynamic = lambda
        .attributes
        .code
        .as_ref()
        .unwrap()
        .bytecode
        .opcodes
        .iter()
        .find_map(|(_, opcode)| match opcode {
            Opcode::InvokeDynamic(value) => Some(value),
            _ => None,
        })
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(bootstrap, &dynamic.bootstrap));
    let annotation = read("ParserVisible.class");
    assert!(
        annotation
            .methods()
            .iter()
            .all(|method| method.attributes.annotation_default.is_some())
    );
    let local = read("ParserAttributes$1Local.class");
    assert!(local.attributes().enclosing_method.is_some());
    read("ParserAttributes$Nested.class");
    read("ParserInvisible.class");
    read("ParserType.class");
}

#[test]
fn parses_javac_debug_tables_and_method_parameters() {
    let dir = TempDir::new();
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/java/ParserAttributes.java");
    let result =
        std::process::Command::new(std::env::var_os("JAVAC").unwrap_or_else(|| "javac".into()))
            .args(["--release", "8", "-g", "-parameters", "-d"])
            .arg(&dir.0)
            .arg(source)
            .output()
            .expect("run javac for attribute fixture");
    assert!(
        result.status.success(),
        "javac: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let bytes = std::fs::read(dir.0.join("ParserAttributes.class")).unwrap();
    let image = crate::class_parser::parse_class(&bytes).unwrap();
    assert!(
        image
            .attributes()
            .source_file
            .as_ref()
            .unwrap()
            .eq_str("ParserAttributes.java")
    );
    let run = image
        .methods()
        .iter()
        .find(|method| method.name == "run")
        .unwrap();
    let parameters = run.attributes.parameters.as_ref().unwrap();
    assert_eq!(parameters.len(), 1);
    assert!(parameters[0].name.as_ref().unwrap().eq_str("input"));
    assert_eq!(parameters[0].access_flags, 0x10);
    let attrs = &run.attributes.code.as_ref().unwrap().attributes;
    assert!(!attrs.line_number_tables.is_empty());
    assert!(
        attrs
            .local_variable_tables
            .iter()
            .flatten()
            .any(|local| local.name.eq_str("input"))
    );
    assert!(
        attrs
            .local_variable_type_tables
            .iter()
            .flatten()
            .any(|local| local.name.eq_str("genericLocal")
                && local.signature.eq_str("Ljava/util/List<TT;>;"))
    );
}
