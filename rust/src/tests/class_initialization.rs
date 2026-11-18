use super::support::{assert_value, fixture_dir, init_vm, isolated, load_normal_klass, run_method};
use crate::{
    class_loader::class_loader_data::ClassLoaderData,
    engines::{
        exec_dispatcher::ExecDispatcher,
        exec_error::{ExecError, ExecErrorKind},
        invocation::Invocation,
        protocol::{DispatchOutcome, ExecBudget, MethodReturn},
    },
    oops::{
        jvalue::{JByte, JChar, JDouble, JFloat, JInt, JLong, JShort, JValue},
        normal_klass::NormalKlass,
        symbol_table::SymbolTable,
    },
    runtime::{class_initialization::InitializationState, java_thread::JavaThread},
};

fn start<'a>(owner: &'a NormalKlass<'a>, method: &str) -> JavaThread<'a, 'a> {
    let mut thread = JavaThread::new();
    ExecDispatcher::start(
        &mut thread,
        Invocation::lookup_declared(owner, method, "()I", &[]).unwrap(),
    )
    .unwrap();
    thread
}

fn finish(thread: &mut JavaThread<'_, '_>, fuel: u64) -> Result<MethodReturn, ExecError> {
    for _ in 0..10000 {
        match ExecDispatcher::poll(thread, &mut ExecBudget::new(fuel))? {
            DispatchOutcome::Yielded => {}
            DispatchOutcome::Completed(value) => return Ok(value),
            DispatchOutcome::BlockedOnInitialization => panic!("unexpected contention"),
        }
    }
    panic!("execution did not complete")
}

#[test]
fn initialization_orders_dependencies_and_retries_active_uses_once() {
    isolated(
        concat!(
            module_path!(),
            "::initialization_orders_dependencies_and_retries_active_uses_once"
        ),
        || {
            init_vm(fixture_dir());
            for fuel in [1, 10000] {
                let cld = ClassLoaderData::new(None);
                let owner =
                    load_normal_klass(&cld, &SymbolTable::intern("ClassInitialization")).unwrap();
                for _ in 0..2 {
                    assert_value(
                        run_method(owner, "order", "()I", &[], fuel),
                        JValue::Int(JInt(212)),
                    );
                }
                assert_value(
                    run_method(owner, "recursive", "()I", &[], fuel),
                    JValue::Int(JInt(309)),
                );
                assert_value(
                    run_method(owner, "cycle", "()I", &[], fuel),
                    JValue::Int(JInt(21)),
                );
                assert_value(
                    run_method(
                        owner,
                        "write",
                        "(J)J",
                        &[JValue::Long(JLong(i64::MIN + 17))],
                        fuel,
                    ),
                    JValue::Long(JLong(i64::MIN + 17)),
                );
                assert_value(
                    run_method(owner, "call", "(J)J", &[JValue::Long(JLong(123))], fuel),
                    JValue::Long(JLong(130)),
                );
                assert_value(
                    run_method(owner, "defaults", "()I", &[], fuel),
                    JValue::Int(JInt(6456)),
                );
                assert_value(
                    run_method(owner, "interfaceOnly", "()I", &[], fuel),
                    JValue::Int(JInt(88)),
                );
                let untouched = load_normal_klass(
                    &cld,
                    &SymbolTable::intern("ClassInitialization$PlainParent"),
                )
                .unwrap();
                assert!(matches!(
                    *untouched.initialization.lock(),
                    InitializationState::Uninitialized
                ));

                let clinit =
                    load_normal_klass(&cld, &SymbolTable::intern("ClassInitialization$Self"))
                        .unwrap();
                assert!(matches!(
                    Invocation::lookup_declared(clinit, "<clinit>", "()V", &[])
                        .err()
                        .unwrap()
                        .kind,
                    ExecErrorKind::UnsupportedInvocation { .. }
                ));

                let direct =
                    load_normal_klass(&cld, &SymbolTable::intern("ClassInitialization$Direct"))
                        .unwrap();
                for _ in 0..2 {
                    assert_value(
                        run_method(direct, "read", "()I", &[], fuel),
                        JValue::Int(JInt(117)),
                    );
                }

                let constants =
                    load_normal_klass(&cld, &SymbolTable::intern("ClassInitialization$Constants"))
                        .unwrap();
                assert_value(
                    run_method(constants, "entry", "()I", &[], fuel),
                    JValue::Int(JInt(42)),
                );
                for (name, desc, expected) in [
                    ("B", "B", JValue::Byte(JByte(-7))),
                    ("C", "C", JValue::Char(JChar(65535))),
                    ("S", "S", JValue::Short(JShort(-300))),
                    ("I", "I", JValue::Int(JInt(42))),
                    ("J", "J", JValue::Long(JLong(-123456789123))),
                    ("F", "F", JValue::Float(JFloat(-0.0))),
                    ("D", "D", JValue::Double(JDouble(2.5))),
                ] {
                    let field = constants.find_declared_field(name, desc).unwrap();
                    assert_value(
                        MethodReturn::Value(constants.fields().load_static(field)),
                        expected,
                    );
                }
            }
        },
    );
}

#[test]
fn initialization_failure_is_persistent_and_propagates_to_dependents() {
    isolated(
        concat!(
            module_path!(),
            "::initialization_failure_is_persistent_and_propagates_to_dependents"
        ),
        || {
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let owner =
                load_normal_klass(&cld, &SymbolTable::intern("ClassInitialization")).unwrap();
            let first = finish(&mut start(owner, "dependencyFailure"), 1)
                .err()
                .unwrap();
            assert!(matches!(first.kind, ExecErrorKind::DivisionByZero));
            for (method, name) in [
                ("failure", "ClassInitialization$Failure"),
                ("dependencyFailure", "ClassInitialization$FailedChild"),
            ] {
                let error = finish(&mut start(owner, method), 1).err().unwrap();
                assert!(
                    matches!(error.kind, ExecErrorKind::InitializationFailed { owner, .. } if owner == name)
                );
            }
            let text =
                load_normal_klass(&cld, &SymbolTable::intern("ClassInitialization$Text")).unwrap();
            let first = finish(&mut start(text, "entry"), 1).err().unwrap();
            assert!(matches!(
                first.kind,
                ExecErrorKind::UnsupportedConstantInitialization { .. }
            ));
            let second = finish(&mut start(text, "entry"), 1).err().unwrap();
            assert!(matches!(
                second.kind,
                ExecErrorKind::InitializationFailed { .. }
            ));
        },
    );
}

#[test]
fn competing_threads_wait_and_abandoned_initialization_fails() {
    isolated(
        concat!(
            module_path!(),
            "::competing_threads_wait_and_abandoned_initialization_fails"
        ),
        || {
            init_vm(fixture_dir());
            for abandon in [false, true] {
                let cld = ClassLoaderData::new(None);
                let owner =
                    load_normal_klass(&cld, &SymbolTable::intern("ClassInitialization")).unwrap();
                let slow =
                    load_normal_klass(&cld, &SymbolTable::intern("ClassInitialization$Slow"))
                        .unwrap();
                let mut first = start(owner, "slow");
                for _ in 0..100 {
                    assert!(matches!(
                        ExecDispatcher::poll(&mut first, &mut ExecBudget::new(1)).unwrap(),
                        DispatchOutcome::Yielded
                    ));
                    if matches!(
                        *slow.initialization.lock(),
                        InitializationState::Initializing(_)
                    ) {
                        break;
                    }
                }
                assert!(
                    matches!(*slow.initialization.lock(), InitializationState::Initializing(id) if id == first.id())
                );
                let mut second = start(owner, "slow");
                assert_ne!(first.id(), second.id());
                assert!(matches!(
                    ExecDispatcher::poll(&mut second, &mut ExecBudget::new(10000)).unwrap(),
                    DispatchOutcome::BlockedOnInitialization
                ));
                if abandon {
                    drop(first);
                    assert!(matches!(
                        finish(&mut second, 1).err().unwrap().kind,
                        ExecErrorKind::InitializationFailed { .. }
                    ));
                } else {
                    assert_value(finish(&mut first, 1).unwrap(), JValue::Int(JInt(4950)));
                    assert_value(finish(&mut second, 1).unwrap(), JValue::Int(JInt(4950)));
                    let count = slow.find_declared_field("count", "I").unwrap();
                    assert!(matches!(
                        slow.fields().load_static(count),
                        JValue::Int(JInt(1))
                    ));
                }
            }
        },
    );
}
