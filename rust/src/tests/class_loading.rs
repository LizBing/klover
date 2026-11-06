use super::support::{
    TempDir, ensure_metaspace, fixture_dir, hierarchy_bytes, init_vm, isolated, load_normal_klass,
};
use crate::{
    class_loader::{class_loader_data::ClassLoaderData, load_error::LoadError},
    oops::{klass::Klass, oops_errors::LinkageError, symbol_table::SymbolTable},
};

#[test]
fn defines_bytes_once_and_preserves_parse_errors() {
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    let bytes = hierarchy_bytes("java/lang/Object", None, &[], 0x21);
    let name = SymbolTable::intern("java/lang/Object");
    let error = cld.define_klass(b"not a class file").unwrap_err();
    assert!(matches!(error, LoadError::Parse(error)
        if matches!(error.kind, crate::class_parser::ParseErrorKind::BadMagic)));
    assert!(cld.find_loaded_klass(&name).is_none());
    let klass = cld.define_klass(&bytes).unwrap();
    assert!(std::ptr::eq(klass, cld.find_loaded_klass(&name).unwrap()));
    assert!(std::ptr::eq(klass, cld.load_klass(&name).unwrap()));
    assert!(matches!(
        cld.define_klass(&bytes),
        Err(LoadError::Duplicated)
    ));
    assert!(std::ptr::eq(klass, cld.find_loaded_klass(&name).unwrap()));
}

#[test]
fn rejects_invalid_hierarchies_without_publishing() {
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    let define = |name, parent, interfaces: &[&str], flags| {
        cld.define_klass(&hierarchy_bytes(name, parent, interfaces, flags))
    };
    define("java/lang/Object", None, &[], 0x21).unwrap();
    define("Base", Some("java/lang/Object"), &[], 0x21).unwrap();
    define("Final", Some("java/lang/Object"), &[], 0x31).unwrap();
    define("Iface", Some("java/lang/Object"), &[], 0x601).unwrap();

    assert!(matches!(
        define("NoSuper", None, &[], 0x21),
        Err(LoadError::Linkage(LinkageError::InvalidSuperclass { .. }))
    ));
    assert!(matches!(define("ExtendsIface", Some("Iface"), &[], 0x21),
        Err(LoadError::Linkage(LinkageError::SuperIsInterface { name })) if name == "Iface"));
    assert!(matches!(define("ExtendsFinal", Some("Final"), &[], 0x21),
        Err(LoadError::Linkage(LinkageError::FinalSuperclass { name })) if name == "Final"));
    assert!(matches!(
        define("BadIfaceSuper", Some("Base"), &[], 0x601),
        Err(LoadError::Linkage(LinkageError::InvalidSuperclass { .. }))
    ));
    assert!(
        matches!(define("ImplementsClass", Some("Base"), &["Base"], 0x21),
        Err(LoadError::Linkage(LinkageError::NotInterface { name })) if name == "Base")
    );
    assert!(matches!(
        define("DuplicateIfaces", Some("Base"), &["Iface", "Iface"], 0x21),
        Err(LoadError::Linkage(LinkageError::InvalidInterfaces { .. }))
    ));
    for name in [
        "NoSuper",
        "ExtendsIface",
        "ExtendsFinal",
        "BadIfaceSuper",
        "ImplementsClass",
        "DuplicateIfaces",
    ] {
        assert!(
            cld.find_loaded_klass(&SymbolTable::intern(name)).is_none(),
            "{name}"
        );
    }
    // A link error must not prevent defining corrected bytes with that name.
    define("ExtendsFinal", Some("Base"), &[], 0x21).unwrap();

    let other = ClassLoaderData::new(None);
    assert!(matches!(
        other.define_klass(&hierarchy_bytes("java/lang/Object", None, &[], 0x601)),
        Err(LoadError::Linkage(LinkageError::InvalidSuperclass { .. }))
    ));
}

#[test]
fn hierarchy_access_errors_do_not_publish_classes_and_allow_retry() {
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    cld.define_klass(&hierarchy_bytes("java/lang/Object", None, &[], 0x21))
        .unwrap();
    for (name, flags) in [
        ("p/HiddenBase", 0x20),
        ("p/PublicBase", 0x21),
        ("p/HiddenIface", 0x600),
        ("p/PublicIface", 0x601),
    ] {
        cld.define_klass(&hierarchy_bytes(name, Some("java/lang/Object"), &[], flags))
            .unwrap();
    }
    for (simple_name, target, replacement, interface, flags) in [
        ("Child", "p/HiddenBase", "p/PublicBase", false, 0x21),
        ("Implementor", "p/HiddenIface", "p/PublicIface", true, 0x21),
        ("SubIface", "p/HiddenIface", "p/PublicIface", true, 0x601),
    ] {
        let bytes = |name: &str, dependency: &str| {
            if interface {
                hierarchy_bytes(name, Some("java/lang/Object"), &[dependency], flags)
            } else {
                hierarchy_bytes(name, Some(dependency), &[], flags)
            }
        };
        let same_package = format!("p/{simple_name}");
        assert!(cld.define_klass(&bytes(&same_package, target)).is_ok());

        let other_package = format!("q/{simple_name}");
        let name = SymbolTable::intern(&other_package);
        let dependency_name = SymbolTable::intern(target);
        let dependency = cld.find_loaded_klass(&dependency_name).unwrap();
        assert!(matches!(cld.define_klass(&bytes(&other_package, target)),
            Err(LoadError::Linkage(LinkageError::IllegalAccess { from, target: actual }))
                if from == other_package && actual == target));
        assert!(cld.find_loaded_klass(&name).is_none());
        assert!(std::ptr::eq(
            dependency,
            cld.find_loaded_klass(&dependency_name).unwrap()
        ));

        let corrected = cld
            .define_klass(&bytes(&other_package, replacement))
            .unwrap();
        assert!(std::ptr::eq(
            corrected,
            cld.find_loaded_klass(&name).unwrap()
        ));
        assert!(std::ptr::eq(corrected, cld.load_klass(&name).unwrap()));
    }
}

#[test]
fn rejects_circular_superclasses_and_interfaces_and_allows_retry() {
    isolated(
        concat!(
            module_path!(),
            "::rejects_circular_superclasses_and_interfaces_and_allows_retry"
        ),
        || {
            let dir = TempDir::new();
            std::fs::write(
                dir.0.join("A.class"),
                hierarchy_bytes("A", Some("B"), &[], 0x21),
            )
            .unwrap();
            std::fs::write(
                dir.0.join("B.class"),
                hierarchy_bytes("B", Some("A"), &[], 0x21),
            )
            .unwrap();
            for (name, dependency) in [("I", "J"), ("J", "I")] {
                std::fs::write(
                    dir.0.join(format!("{name}.class")),
                    hierarchy_bytes(name, Some("java/lang/Object"), &[dependency], 0x601),
                )
                .unwrap();
            }
            init_vm(dir.0.clone());
            let cld = ClassLoaderData::new(None);
            cld.define_klass(&hierarchy_bytes("java/lang/Object", None, &[], 0x21))
                .unwrap();
            for name in ["A", "I"] {
                assert!(matches!(cld.load_klass(&SymbolTable::intern(name)),
                Err(LoadError::Linkage(LinkageError::ClassCircularity { name: actual })) if actual == name));
            }
            for name in ["A", "B", "I", "J"] {
                assert!(cld.find_loaded_klass(&SymbolTable::intern(name)).is_none());
            }
            std::fs::write(
                dir.0.join("B.class"),
                hierarchy_bytes("B", Some("java/lang/Object"), &[], 0x21),
            )
            .unwrap();
            assert!(cld.load_klass(&SymbolTable::intern("A")).is_ok());
            std::fs::write(
                dir.0.join("J.class"),
                hierarchy_bytes("J", Some("java/lang/Object"), &[], 0x601),
            )
            .unwrap();
            assert!(cld.load_klass(&SymbolTable::intern("I")).is_ok());
        },
    );
}

#[test]
fn rejects_wrong_class_names_before_publication() {
    isolated(
        concat!(
            module_path!(),
            "::rejects_wrong_class_names_before_publication"
        ),
        || {
            let dir = TempDir::new();
            let path = dir.0.join("Expected.class");
            std::fs::write(
                &path,
                hierarchy_bytes("Actual", Some("java/lang/Object"), &[], 0x21),
            )
            .unwrap();
            init_vm(dir.0.clone());
            let cld = ClassLoaderData::new(None);
            let expected = SymbolTable::intern("Expected");
            assert!(
                matches!(cld.load_klass(&expected), Err(LoadError::NameMismatch { expected, actual })
            if expected == "Expected" && actual == "Actual")
            );
            assert!(cld.find_loaded_klass(&expected).is_none());
            assert!(
                cld.find_loaded_klass(&SymbolTable::intern("Actual"))
                    .is_none()
            );
            cld.define_klass(&hierarchy_bytes("java/lang/Object", None, &[], 0x21))
                .unwrap();
            std::fs::write(
                &path,
                hierarchy_bytes("Expected", Some("java/lang/Object"), &[], 0x21),
            )
            .unwrap();
            assert!(cld.load_klass(&expected).is_ok());
        },
    );
}

#[test]
fn concurrent_definitions_publish_exactly_one_identity() {
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    let bytes = hierarchy_bytes("java/lang/Object", None, &[], 0x21);
    let barrier = std::sync::Barrier::new(8);
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    cld.define_klass(&bytes)
                })
            })
            .collect();
        let mut defined = 0;
        for worker in workers {
            match worker.join().unwrap() {
                Ok(klass) => {
                    defined += 1;
                    assert!(std::ptr::eq(
                        klass,
                        cld.find_loaded_klass(klass.name()).unwrap()
                    ));
                }
                Err(LoadError::Duplicated) => {}
                result => panic!("unexpected definition result: {result:?}"),
            }
        }
        assert_eq!(defined, 1);
    });
}

#[test]
fn concurrent_loads_reuse_one_hierarchy() {
    isolated(
        concat!(module_path!(), "::concurrent_loads_reuse_one_hierarchy"),
        || {
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let name = SymbolTable::intern("InterfaceLoading$Base");
            let barrier = std::sync::Barrier::new(8);
            std::thread::scope(|scope| {
                let workers: Vec<_> = (0..8)
                    .map(|_| {
                        scope.spawn(|| {
                            barrier.wait();
                            cld.load_klass(&name).unwrap()
                        })
                    })
                    .collect();
                let loaded: Vec<_> = workers
                    .into_iter()
                    .map(|worker| worker.join().unwrap())
                    .collect();
                assert!(loaded.iter().all(|klass| std::ptr::eq(*klass, loaded[0])));
                let parent = cld
                    .load_klass(&SymbolTable::intern("java/lang/Object"))
                    .unwrap();
                assert!(std::ptr::eq(
                    loaded[0].super_klass().unwrap(),
                    parent.as_normal().unwrap()
                ));
            });
        },
    );
}

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
            let cld = ClassLoaderData::new(None);
            let name = SymbolTable::intern("InterfaceLoading$Base");
            let error = cld.load_klass(&name).unwrap_err();
            assert!(matches!(error, LoadError::NoClassDefFound));
            std::fs::copy(
                fixtures.join("InterfaceLoading$Second.class"),
                dir.0.join("InterfaceLoading$Second.class"),
            )
            .unwrap();
            assert!(
                cld.load_klass(&name).is_ok(),
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
            let cld = ClassLoaderData::new(None);
            let name = SymbolTable::intern("Cases");
            let first = cld.load_klass(&name).unwrap();
            let second = cld.load_klass(&name).unwrap();
            assert!(
                std::ptr::eq(first, second),
                "repeated lookup created a new class"
            );
            let Klass::Normal(klass) = first else {
                panic!("expected a normal class")
            };
            assert_eq!(klass.name().utf8(), "Cases");
            assert!(klass.find_declared_method("sum", "(I)I").is_some());
            assert!(load_normal_klass(&cld, &SymbolTable::intern("java/lang/Object")).is_ok());
        },
    );
}

#[test]
fn missing_and_malformed_classes_report_errors() {
    isolated(
        concat!(
            module_path!(),
            "::missing_and_malformed_classes_report_errors"
        ),
        || {
            let dir = TempDir::new();
            std::fs::write(dir.0.join("Broken.class"), b"not a class file").unwrap();
            init_vm(dir.0.clone());
            let cld = ClassLoaderData::new(None);
            let error = cld.load_klass(&SymbolTable::intern("Missing")).unwrap_err();
            assert!(matches!(error, LoadError::NotFound));
            let error = cld.load_klass(&SymbolTable::intern("Broken")).unwrap_err();
            assert!(matches!(error, LoadError::Parse(_)));
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
            let cld = ClassLoaderData::new(None);
            let name = SymbolTable::intern("Cases");
            let error = cld.load_klass(&name).unwrap_err();
            assert!(matches!(error, LoadError::NoClassDefFound));
            std::fs::create_dir_all(dir.0.join("java/lang")).unwrap();
            std::fs::copy(
                fixtures.join("java/lang/Object.class"),
                dir.0.join("java/lang/Object.class"),
            )
            .unwrap();
            assert!(
                cld.load_klass(&name).is_ok(),
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
