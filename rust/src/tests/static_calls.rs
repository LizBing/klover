use super::support::{assert_value, fixture_dir, init_vm, isolated, load_normal_klass, run_method};
use crate::{
    class_loader::class_loader_data::ClassLoaderData,
    engines::{
        exec_dispatcher::ExecDispatcher,
        exec_error::{ExecError, ExecErrorKind},
        invocation::Invocation,
        protocol::{DispatchOutcome, ExecBudget, MethodReturn},
    },
    gc_bindings::oop_hierarchy::NObjPtr,
    oops::{
        jvalue::JValue::{self, *},
        normal_klass::NormalKlass,
        oops_errors::ResolveError,
        symbol_table::SymbolTable,
    },
    runtime::java_thread::JavaThread,
};

fn execution_error(
    owner: &NormalKlass<'_>,
    name: &str,
    desc: &str,
    args: &[JValue],
    fuel: u64,
) -> ExecError {
    let mut thread = JavaThread::new();
    ExecDispatcher::start(
        &mut thread,
        Invocation::lookup_declared(owner, name, desc, args).unwrap(),
    )
    .unwrap();
    for _ in 0..1000 {
        match ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(fuel)) {
            Err(error) => return error,
            Ok(DispatchOutcome::Yielded) => (),
            Ok(DispatchOutcome::Completed(_)) => panic!("{name} should fail"),
        }
    }
    panic!("{name} did not report an error")
}

#[test]
fn invokestatic_java_calls_cover_resolution_arguments_and_returns() {
    isolated(
        concat!(
            module_path!(),
            "::invokestatic_java_calls_cover_resolution_arguments_and_returns"
        ),
        || {
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let owner = load_normal_klass(&cld, &SymbolTable::intern("StaticCalls")).unwrap();
            let null = Ref(NObjPtr::null());
            let cases: &[(&str, &str, &[JValue], JValue)] = &[
                ("sameClass", "(II)I", &[Int(7), Int(12)], Int(95)),
                ("repeated", "(II)I", &[Int(7), Int(12)], Int(-10)),
                ("crossClass", "(II)I", &[Int(7), Int(12)], Int(82)),
                ("inherited", "(I)I", &[Int(9)], Int(14)),
                ("interfaceCall", "(I)I", &[Int(-7)], Int(-21)),
                ("overloaded", "(I)I", &[Int(i32::MIN)], Int(i32::MIN)),
                (
                    "overloaded",
                    "(J)J",
                    &[Long(i64::MIN + 123)],
                    Long(i64::MIN + 123),
                ),
                (
                    "mixed",
                    "(IJDFLjava/lang/Object;)J",
                    &[Int(3), Long(1 << 40), Double(-2.75), Float(4.5), null],
                    Long((1 << 40) + 112),
                ),
                ("floatCall", "(F)F", &[Float(-0.0)], Float(-0.0)),
                ("floatCall", "(F)F", &[Float(f32::NAN)], Float(f32::NAN)),
                ("doubleCall", "(D)D", &[Double(-0.0)], Double(-0.0)),
                (
                    "doubleCall",
                    "(D)D",
                    &[Double(f64::INFINITY)],
                    Double(f64::INFINITY),
                ),
                (
                    "referenceCall",
                    "(Ljava/lang/Object;)Ljava/lang/Object;",
                    &[null],
                    null,
                ),
                ("arrayCall", "([I)[I", &[null], null),
                ("narrow", "(I)I", &[Int(65535)], Int(65534)),
                ("byteReturn", "(I)I", &[Int(255)], Int(-1)),
                ("shortReturn", "(I)I", &[Int(65535)], Int(-1)),
                ("charReturn", "(I)I", &[Int(-1)], Int(65535)),
                ("booleanReturn", "(Z)Z", &[Boolean(1)], Int(1)),
                ("booleanReturn", "(Z)Z", &[Boolean(0)], Int(0)),
                ("voidCall", "(I)I", &[Int(33)], Int(33)),
                ("recursive", "(I)I", &[Int(20)], Int(210)),
                ("mutual", "(I)I", &[Int(20)], Int(20)),
            ];
            for &(name, desc, args, expected) in cases {
                // Reuse the same metadata to exercise repeated calls and resolved caches.
                for fuel in [1, 2, 3, 10000] {
                    assert_value(run_method(owner, name, desc, args, fuel), expected);
                }
            }
            for fuel in [1, 10000] {
                assert!(matches!(
                    run_method(owner, "onlyVoid", "()V", &[], fuel),
                    MethodReturn::Void
                ));
            }
        },
    );
}

#[test]
fn invokestatic_budget_boundaries_preserve_caller_and_callee() {
    isolated(
        concat!(
            module_path!(),
            "::invokestatic_budget_boundaries_preserve_caller_and_callee"
        ),
        || {
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let owner = load_normal_klass(&cld, &SymbolTable::intern("StaticCalls")).unwrap();
            let mut thread = JavaThread::new();
            ExecDispatcher::start(
                &mut thread,
                Invocation::lookup_declared(owner, "accessProbe", "()I", &[]).unwrap(),
            )
            .unwrap();
            // The entry is invokestatic; ireturn. The callee is bipush 11; ireturn.
            for fuel in [0, 1, 1, 1] {
                let mut budget = ExecBudget::new(fuel);
                assert!(matches!(
                    ExecDispatcher::poll(&mut thread, &mut budget).unwrap(),
                    DispatchOutcome::Yielded
                ));
                assert_eq!(budget.remaining(), 0);
                assert!(thread.is_runnable());
            }
            assert!(matches!(
                ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(1)).unwrap(),
                DispatchOutcome::Completed(MethodReturn::Value(Int(11)))
            ));
            assert!(thread.is_terminated());
            assert!(thread.current_frame().is_none());
        },
    );
}

#[test]
fn invokestatic_reports_unsupported_targets_and_callee_context() {
    isolated(
        concat!(
            module_path!(),
            "::invokestatic_reports_unsupported_targets_and_callee_context"
        ),
        || {
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let owner = load_normal_klass(&cld, &SymbolTable::intern("StaticCalls")).unwrap();
            for fuel in [1, 10000] {
                for name in ["nativeProbe", "synchronizedProbe", "initializationProbe"] {
                    let error = execution_error(owner, name, "()I", &[], fuel);
                    match name {
                        "initializationProbe" => assert!(
                            matches!(error.kind, ExecErrorKind::UnsupportedClassInitialization { owner } if owner == "StaticInitialized")
                        ),
                        _ => assert!(matches!(
                            error.kind,
                            ExecErrorKind::UnsupportedInvocation { .. }
                        )),
                    }
                    let brief = error.brief.unwrap();
                    assert_eq!(brief.klass, "StaticCalls");
                    assert_eq!(brief.method, name);
                    assert_eq!(brief.instruction_index, Some(0));
                }
                let error = execution_error(owner, "divide", "(II)I", &[Int(1), Int(0)], fuel);
                assert!(matches!(error.kind, ExecErrorKind::DivisionByZero));
                let brief = error.brief.unwrap();
                assert_eq!(brief.klass, "StaticCallee");
                assert_eq!(brief.method, "divide");
                assert_eq!(brief.instruction_index, Some(2));
            }
        },
    );
}

#[test]
fn invokestatic_enforces_access_and_invocation_kind_from_classfiles() {
    isolated(
        concat!(
            module_path!(),
            "::invokestatic_enforces_access_and_invocation_kind_from_classfiles"
        ),
        || {
            init_vm(fixture_dir());
            let original = std::fs::read(fixture_dir().join("StaticCalls.class")).unwrap();
            // Change just the CONSTANT_Utf8 method name in the caller. This creates
            // references javac would refuse to produce; the descriptor stays ()I.
            let needle = b"\x01\x00\x07allowed";
            let offsets: Vec<_> = original
                .windows(needle.len())
                .enumerate()
                .filter_map(|(i, bytes)| (bytes == needle).then_some(i))
                .collect();
            assert_eq!(offsets.len(), 1);
            for replacement in ["blocked", "regular", "missing"] {
                let mut bytes = original.clone();
                let start = offsets[0] + 3;
                bytes[start..start + 7].copy_from_slice(replacement.as_bytes());
                let cld = ClassLoaderData::new(None);
                let owner = cld.define_klass(&bytes).unwrap().as_normal().unwrap();
                for fuel in [1, 10000] {
                    let error = execution_error(owner, "accessProbe", "()I", &[], fuel);
                    match replacement {
                        "blocked" => assert!(matches!(
                            error.kind,
                            ExecErrorKind::Resolve(ResolveError::IllegalMethodAccess { .. })
                        )),
                        "regular" => assert!(matches!(
                            error.kind,
                            ExecErrorKind::ExpectedStaticMethod { .. }
                        )),
                        "missing" => assert!(matches!(
                            error.kind,
                            ExecErrorKind::Resolve(ResolveError::NoSuchMethod { .. })
                        )),
                        _ => unreachable!(),
                    }
                    let brief = error.brief.unwrap();
                    assert_eq!(brief.klass, "StaticCalls");
                    assert_eq!(brief.method, "accessProbe");
                    assert_eq!(brief.instruction_index, Some(0));
                }
            }
        },
    );
}

#[test]
fn invokestatic_lowering_preserves_shared_references_overloads_and_interface_tags() {
    isolated(
        concat!(
            module_path!(),
            "::invokestatic_lowering_preserves_shared_references_overloads_and_interface_tags"
        ),
        || {
            use crate::code::instructions::Instruction;
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let owner = load_normal_klass(&cld, &SymbolTable::intern("StaticCalls")).unwrap();
            let refs = |name, desc| {
                owner
                    .find_declared_method(name, desc)
                    .unwrap()
                    .code
                    .as_ref()
                    .unwrap()
                    .insts
                    .iter()
                    .filter_map(|inst| match inst {
                        Instruction::InvokeStatic(reference) => Some(*reference),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            };
            let repeated = refs("repeated", "(II)I");
            assert_eq!(repeated.len(), 2);
            assert!(std::ptr::eq(repeated[0], repeated[1]));
            let ints = refs("overloaded", "(I)I");
            let longs = refs("overloaded", "(J)J");
            assert_eq!((ints.len(), longs.len()), (1, 1));
            assert!(!std::ptr::eq(ints[0], longs[0]));
            assert_eq!(
                ints[0].resolve(owner).unwrap().method.desc.raw().utf8(),
                "(I)I"
            );
            assert_eq!(
                longs[0].resolve(owner).unwrap().method.desc.raw().utf8(),
                "(J)J"
            );
            let interface = refs("interfaceCall", "(I)I");
            assert_eq!(interface.len(), 1);
            assert!(
                interface[0]
                    .resolve(owner)
                    .unwrap()
                    .declaring_klass
                    .is_interface()
            );
        },
    );
}

#[test]
fn invokestatic_failure_cannot_resume_past_the_rejected_call() {
    isolated(
        concat!(
            module_path!(),
            "::invokestatic_failure_cannot_resume_past_the_rejected_call"
        ),
        || {
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let owner = load_normal_klass(&cld, &SymbolTable::intern("StaticCalls")).unwrap();
            for fuel in [1, 10000] {
                let mut thread = JavaThread::new();
                ExecDispatcher::start(
                    &mut thread,
                    Invocation::lookup_declared(owner, "retryProbe", "()I", &[]).unwrap(),
                )
                .unwrap();
                let error = ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(fuel))
                    .err()
                    .unwrap();
                assert!(matches!(
                    error.kind,
                    ExecErrorKind::UnsupportedInvocation { .. }
                ));
                let brief = error.brief.unwrap();
                assert_eq!(brief.klass, "StaticCalls");
                assert_eq!(brief.method, "retryProbe");
                assert_eq!(brief.instruction_index, Some(0));
                assert!(thread.is_terminated());
                // This method returns 42 immediately after the failed native call.
                // Repeated polling must reject it instead of skipping that call.
                for retry_fuel in [0, 1, 10000] {
                    let mut budget = ExecBudget::new(retry_fuel);
                    let error = ExecDispatcher::poll(&mut thread, &mut budget)
                        .err()
                        .expect("terminated thread resumed the caller");
                    assert!(matches!(error.kind, ExecErrorKind::ThreadTerminated));
                    assert_eq!(budget.remaining(), retry_fuel);
                    assert!(thread.is_terminated());
                }
            }
        },
    );
}
