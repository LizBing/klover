use std::sync::Arc;

use super::invoke_static;
use crate::{
    class_loader::class_loader_data::ClassLoaderData,
    class_parser::{ClassRef, MethodDescriptor, MethodMemRef, MethodMemRefKind},
    engines::{
        exec_error::ExecErrorKind,
        interpreter::{interpreter_frame::InterpreterFrame, slot::Slot, step_outcome::StepControl},
        invocation::Invocation,
    },
    gc_bindings::oop_hierarchy::NObjPtr,
    oops::{
        constant_entries::MethodRef, jvalue::JValue, normal_klass::NormalKlass,
        oops_errors::ResolveError,
    },
    tests::support::ensure_metaspace,
};

// Small explicit class files let these tests exercise access combinations that
// javac deliberately refuses to emit, without relying on Java exception support.
fn define<'a>(
    cld: &'a ClassLoaderData,
    name: &str,
    parent: Option<&str>,
    interfaces: &[&str],
    flags: u16,
    methods: &[(&str, &str, u16)],
) -> &'a NormalKlass<'a> {
    fn u2(out: &mut Vec<u8>, value: u16) {
        out.extend_from_slice(&value.to_be_bytes());
    }
    fn utf8(out: &mut Vec<u8>, value: &str) {
        out.push(1);
        u2(out, value.len() as u16);
        out.extend_from_slice(value.as_bytes());
    }
    let names: Vec<_> = std::iter::once(name)
        .chain(parent)
        .chain(interfaces.iter().copied())
        .collect();
    let mut out = vec![0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 52];
    let code_index = 1 + 2 * names.len() as u16;
    u2(&mut out, code_index + 1 + 2 * methods.len() as u16);
    for (index, name) in names.iter().enumerate() {
        utf8(&mut out, name);
        out.push(7);
        u2(&mut out, 1 + index as u16 * 2);
    }
    utf8(&mut out, "Code");
    for (name, desc, _) in methods {
        utf8(&mut out, name);
        utf8(&mut out, desc);
    }
    for value in [
        flags,
        2,
        if parent.is_some() { 4 } else { 0 },
        interfaces.len() as u16,
    ] {
        u2(&mut out, value);
    }
    for index in 0..interfaces.len() {
        u2(
            &mut out,
            (if parent.is_some() { 6 } else { 4 }) + index as u16 * 2,
        );
    }
    u2(&mut out, 0); // fields
    u2(&mut out, methods.len() as u16);
    for (index, (_, _, flags)) in methods.iter().enumerate() {
        let has_code = flags & (0x100 | 0x400) == 0;
        for value in [
            *flags,
            code_index + 1 + index as u16 * 2,
            code_index + 2 + index as u16 * 2,
            u16::from(has_code),
        ] {
            u2(&mut out, value);
        }
        if has_code {
            u2(&mut out, code_index);
            out.extend_from_slice(&13u32.to_be_bytes());
            u2(&mut out, 32);
            u2(&mut out, 32);
            out.extend_from_slice(&1u32.to_be_bytes());
            out.push(0xb1);
            u2(&mut out, 0);
            u2(&mut out, 0);
        }
    }
    u2(&mut out, 0);
    cld.define_klass(&out).unwrap().as_normal().unwrap()
}

fn root(cld: &ClassLoaderData) {
    ensure_metaspace();
    define(cld, "java/lang/Object", None, &[], 0x21, &[]);
}

fn class<'a>(
    cld: &'a ClassLoaderData,
    name: &str,
    parent: &str,
    flags: u16,
    methods: &[(&str, &str, u16)],
) -> &'a NormalKlass<'a> {
    define(cld, name, Some(parent), &[], flags, methods)
}

fn reference<'a>(owner: &str, name: &str, desc: &str, interface: bool) -> MethodRef<'a> {
    MethodRef::build(&MethodMemRef {
        kind: if interface {
            MethodMemRefKind::Interface
        } else {
            MethodMemRefKind::Class
        },
        owner: Arc::new(ClassRef { name: owner.into() }),
        name: name.into(),
        descriptor: MethodDescriptor::parse(desc).unwrap(),
    })
}

fn frame<'exec, 'meta: 'exec>(caller: &'exec NormalKlass<'meta>) -> InterpreterFrame<'exec, 'meta> {
    InterpreterFrame::try_new(Invocation::lookup_declared(caller, "caller", "()V", &[]).unwrap())
        .unwrap()
}

#[test]
fn transfers_arguments_in_descriptor_order_without_narrowing() {
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    root(&cld);
    let owner = class(
        &cld,
        "p/Caller",
        "java/lang/Object",
        0x21,
        &[
            ("caller", "()V", 9),
            ("target", "(IZBSCJDFLjava/lang/Object;[I[[D)V", 9),
        ],
    );
    let mut f = frame(owner);
    f.push_int(99); // unrelated caller stack value
    for value in [7, 2, 256, 65536, -1] {
        f.push_int(value);
    }
    f.push_long(i64::MIN + 123);
    f.push_double(-0.0);
    f.push_float(1.5);
    for _ in 0..3 {
        f.push_ref(NObjPtr::null());
    }
    let mref = reference(
        "p/Caller",
        "target",
        "(IZBSCJDFLjava/lang/Object;[I[[D)V",
        false,
    );
    let StepControl::Invoke(call) = invoke_static(&mut f, &mref).unwrap() else {
        panic!("expected call")
    };
    assert!(matches!(f.operand_stack(), [Slot::Int(99)]));
    let callee = InterpreterFrame::try_new(call).unwrap();
    for (index, expected) in [7, 2, 256, 65536, -1].into_iter().enumerate() {
        assert!(
            matches!(callee.get_local(index).unwrap(), Slot::Int(actual) if actual == expected)
        );
    }
    assert!(matches!(callee.get_local(5).unwrap(), Slot::LongHigh(_)));
    assert!(matches!(callee.get_local(6).unwrap(), Slot::LongLow(_)));
    assert!(matches!(
        callee.get_local(7).unwrap(),
        Slot::DoubleHigh([0x80, 0, 0, 0])
    ));
    assert!(matches!(
        callee.get_local(8).unwrap(),
        Slot::DoubleLow([0, 0, 0, 0])
    ));
    assert!(matches!(callee.get_local(9).unwrap(), Slot::Float(1.5)));
    for index in 10..13 {
        assert!(matches!(callee.get_local(index).unwrap(), Slot::Ref(value) if value.is_null()));
    }
}

#[test]
fn access_checks_include_cached_targets_and_inherited_declarations() {
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    root(&cld);
    let base = class(
        &cld,
        "p/Base",
        "java/lang/Object",
        0x21,
        &[
            ("caller", "()V", 9),
            ("publicMethod", "()V", 9),
            ("privateMethod", "()V", 10),
            ("packageMethod", "()V", 8),
            ("protectedMethod", "()V", 12),
        ],
    );
    let peer = class(
        &cld,
        "p/Peer",
        "java/lang/Object",
        0x21,
        &[("caller", "()V", 9)],
    );
    let child = class(&cld, "q/Child", "p/Base", 0x21, &[("caller", "()V", 9)]);
    let stranger = class(
        &cld,
        "q/Stranger",
        "java/lang/Object",
        0x21,
        &[("caller", "()V", 9)],
    );
    for (name, allowed) in [
        ("publicMethod", [true, true, true, true]),
        ("privateMethod", [true, false, false, false]),
        ("packageMethod", [true, true, false, false]),
        ("protectedMethod", [true, true, true, false]),
    ] {
        let mref = reference("p/Base", name, "()V", false);
        // Base resolves first, so subsequent callers exercise the cache path.
        for (caller, allowed) in [base, peer, child, stranger].into_iter().zip(allowed) {
            let mut f = frame(caller);
            f.push_int(17);
            f.next_inst();
            match invoke_static(&mut f, &mref) {
                Ok(StepControl::Invoke(call)) if allowed => {
                    let (target, args) = call.into_parts();
                    assert!(std::ptr::eq(target.declaring_class, base));
                    assert!(args.is_empty());
                }
                Err(error) if !allowed => {
                    assert!(matches!(
                        error.kind,
                        ExecErrorKind::Resolve(ResolveError::IllegalMethodAccess { .. })
                    ));
                    let brief = error.brief.unwrap();
                    assert_eq!(brief.klass, caller.name().utf8());
                    assert_eq!(brief.instruction_index, Some(0));
                }
                _ => panic!(
                    "unexpected access result for {name} from {}",
                    caller.name().utf8()
                ),
            }
            assert!(matches!(f.operand_stack(), [Slot::Int(17)]));
        }
    }
    let mref = reference("q/Child", "protectedMethod", "()V", false);
    let StepControl::Invoke(call) = invoke_static(&mut frame(child), &mref).unwrap() else {
        panic!("expected inherited call")
    };
    assert!(std::ptr::eq(call.into_parts().0.declaring_class, base));
    let nested = class(
        &cld,
        "p/Base$Nested",
        "java/lang/Object",
        0x21,
        &[("caller", "()V", 9)],
    );
    assert!(matches!(
        invoke_static(
            &mut frame(nested),
            &reference("p/Base", "privateMethod", "()V", false)
        )
        .err()
        .unwrap()
        .kind,
        ExecErrorKind::Resolve(ResolveError::IllegalMethodAccess { .. })
    ));
}

#[test]
fn checks_symbolic_owner_access_but_allows_public_methods_from_hidden_bases() {
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    root(&cld);
    let hidden = class(
        &cld,
        "p/Hidden",
        "java/lang/Object",
        0x20,
        &[("caller", "()V", 9), ("target", "()V", 9)],
    );
    class(&cld, "p/PublicChild", "p/Hidden", 0x21, &[]);
    let peer = class(
        &cld,
        "p/Peer",
        "java/lang/Object",
        0x21,
        &[("caller", "()V", 9)],
    );
    let outsider = class(
        &cld,
        "q/Outsider",
        "java/lang/Object",
        0x21,
        &[("caller", "()V", 9)],
    );
    let mref = reference("p/Hidden", "target", "()V", false);
    assert!(invoke_static(&mut frame(peer), &mref).is_ok());
    assert!(matches!(
        invoke_static(&mut frame(outsider), &mref)
            .err()
            .unwrap()
            .kind,
        ExecErrorKind::Resolve(ResolveError::IllegalClassAccess { .. })
    ));
    let StepControl::Invoke(call) = invoke_static(
        &mut frame(outsider),
        &reference("p/PublicChild", "target", "()V", false),
    )
    .unwrap() else {
        panic!("expected call")
    };
    assert!(std::ptr::eq(call.into_parts().0.declaring_class, hidden));
}

#[test]
fn loaders_are_part_of_package_and_class_identity() {
    ensure_metaspace();
    let first = ClassLoaderData::new(Some("same".into()));
    let second = ClassLoaderData::new(Some("same".into()));
    for cld in [&first, &second] {
        root(cld);
        class(
            cld,
            "p/Base",
            "java/lang/Object",
            0x21,
            &[("caller", "()V", 9), ("target", "()V", 8)],
        );
    }
    let a = first
        .find_loaded_klass(&crate::oops::symbol_table::SymbolTable::intern("p/Base"))
        .unwrap()
        .as_normal()
        .unwrap();
    let b = second
        .find_loaded_klass(&crate::oops::symbol_table::SymbolTable::intern("p/Base"))
        .unwrap()
        .as_normal()
        .unwrap();
    assert!(!a.same_runtime_package(b));
    assert!(!a.is_subclass_of(b));
    let mref = reference("p/Base", "target", "()V", false);
    for caller in [a, b] {
        let StepControl::Invoke(call) = invoke_static(&mut frame(caller), &mref).unwrap() else {
            panic!("expected call")
        };
        assert!(std::ptr::eq(call.into_parts().0.declaring_class, caller));
    }
}

#[test]
fn rejects_bad_arguments_and_unsupported_targets_without_mutating_stack() {
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    root(&cld);
    let owner = class(
        &cld,
        "p/Calls",
        "java/lang/Object",
        0x21,
        &[
            ("caller", "()V", 9),
            ("args", "(IJ)V", 9),
            ("instance", "()V", 1),
            ("nativeMethod", "()V", 0x109),
            ("synchronizedMethod", "()V", 0x29),
        ],
    );
    for (name, desc, slots) in [
        ("args", "(IJ)V", vec![Slot::Int(1)]),
        (
            "args",
            "(IJ)V",
            vec![
                Slot::Float(1.0),
                Slot::LongHigh([0; 4]),
                Slot::LongLow([0; 4]),
            ],
        ),
        (
            "args",
            "(IJ)V",
            vec![Slot::Int(1), Slot::LongLow([0; 4]), Slot::LongHigh([0; 4])],
        ),
        ("instance", "()V", vec![Slot::Int(1)]),
        ("nativeMethod", "()V", vec![Slot::Int(1)]),
        ("synchronizedMethod", "()V", vec![Slot::Int(1)]),
        ("missing", "()V", vec![Slot::Int(1)]),
    ] {
        let mut f = frame(owner);
        for slot in &slots {
            f.push(*slot);
        }
        let error = invoke_static(&mut f, &reference("p/Calls", name, desc, false))
            .err()
            .expect("must fail");
        match name {
            "args" => assert!(matches!(
                error.kind,
                ExecErrorKind::StackUnderflow | ExecErrorKind::MismatchSlotType
            )),
            "instance" => assert!(matches!(
                error.kind,
                ExecErrorKind::ExpectedStaticMethod { .. }
            )),
            "missing" => assert!(matches!(
                error.kind,
                ExecErrorKind::Resolve(ResolveError::NoSuchMethod { .. })
            )),
            _ => assert!(matches!(
                error.kind,
                ExecErrorKind::UnsupportedInvocation { .. }
            )),
        }
        assert_eq!(format!("{:?}", f.operand_stack()), format!("{slots:?}"));
    }
}

#[test]
fn rejects_required_initialization_and_handles_interface_static_calls() {
    ensure_metaspace();
    let cld = ClassLoaderData::new(None);
    root(&cld);
    let caller = class(
        &cld,
        "p/Caller",
        "java/lang/Object",
        0x21,
        &[("caller", "()V", 9)],
    );
    class(
        &cld,
        "p/InitBase",
        "java/lang/Object",
        0x21,
        &[("<clinit>", "()V", 8), ("inherited", "()V", 9)],
    );
    class(
        &cld,
        "p/InitChild",
        "p/InitBase",
        0x21,
        &[("target", "()V", 9)],
    );
    define(
        &cld,
        "p/InitInterface",
        Some("java/lang/Object"),
        &[],
        0x601,
        &[("<clinit>", "()V", 8), ("defaultMethod", "()V", 1)],
    );
    define(
        &cld,
        "p/DefaultChild",
        Some("java/lang/Object"),
        &["p/InitInterface"],
        0x21,
        &[("target", "()V", 9)],
    );
    define(
        &cld,
        "p/I",
        Some("java/lang/Object"),
        &[],
        0x601,
        &[("target", "()V", 9)],
    );
    define(
        &cld,
        "p/SubI",
        Some("java/lang/Object"),
        &["p/InitInterface"],
        0x601,
        &[("target", "()V", 9)],
    );
    for (owner, expected) in [
        ("p/InitBase", "p/InitBase"),
        ("p/InitChild", "p/InitBase"),
        ("p/DefaultChild", "p/InitInterface"),
    ] {
        let name = if owner == "p/InitBase" {
            "inherited"
        } else {
            "target"
        };
        let mut f = frame(caller);
        f.push_int(3);
        let error = invoke_static(&mut f, &reference(owner, name, "()V", false))
            .err()
            .unwrap();
        assert!(
            matches!(error.kind, ExecErrorKind::UnsupportedClassInitialization { owner } if owner == expected)
        );
        assert!(matches!(f.operand_stack(), [Slot::Int(3)]));
    }
    for owner in ["p/I", "p/SubI"] {
        assert!(matches!(
            invoke_static(&mut frame(caller), &reference(owner, "target", "()V", true)).unwrap(),
            StepControl::Invoke(_)
        ));
    }
    assert!(matches!(
        invoke_static(
            &mut frame(caller),
            &reference("p/InitBase", "<clinit>", "()V", false)
        )
        .err()
        .unwrap()
        .kind,
        ExecErrorKind::UnsupportedInvocation { .. }
    ));
}
