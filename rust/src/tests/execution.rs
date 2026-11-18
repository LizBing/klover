use super::support::{assert_value, fixture_dir, init_vm, isolated, load_normal_klass, run_method};
use crate::{
    class_loader::class_loader_data::ClassLoaderData,
    engines::{
        exec_dispatcher::ExecDispatcher,
        exec_error::ExecErrorKind,
        invocation::Invocation,
        protocol::{ExecBudget, MethodReturn},
    },
    gc_bindings::oop_hierarchy::NObjPtr,
    oops::{
        jvalue::{
            JDouble, JFloat, JInt, JLong,
            JValue::{self, *},
        },
        symbol_table::SymbolTable,
    },
    runtime::java_thread::JavaThread,
};

#[test]
fn java_arithmetic_and_wide_arguments() {
    isolated(
        concat!(module_path!(), "::java_arithmetic_and_wide_arguments"),
        || {
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let owner = load_normal_klass(&cld, &SymbolTable::intern("Cases")).unwrap();
            let cases: &[(&str, &str, &[JValue], JValue)] = &[
                (
                    "subtract",
                    "(II)I",
                    &[Int(JInt(7)), Int(JInt(12))],
                    Int(JInt(-5)),
                ),
                (
                    "add",
                    "(II)I",
                    &[Int(JInt(i32::MAX)), Int(JInt(1))],
                    Int(JInt(i32::MIN)),
                ),
                (
                    "multiply",
                    "(JJ)J",
                    &[Long(JLong(1 << 40)), Long(JLong(-3))],
                    Long(JLong(-3_298_534_883_328)),
                ),
                (
                    "divide",
                    "(II)I",
                    &[Int(JInt(i32::MIN)), Int(JInt(-1))],
                    Int(JInt(i32::MIN)),
                ),
                (
                    "divide",
                    "(II)I",
                    &[Int(JInt(-7)), Int(JInt(3))],
                    Int(JInt(-2)),
                ),
                (
                    "remainder",
                    "(II)I",
                    &[Int(JInt(-7)), Int(JInt(3))],
                    Int(JInt(-1)),
                ),
                (
                    "shift",
                    "(II)I",
                    &[Int(JInt(-1)), Int(JInt(33))],
                    Int(JInt(i32::MAX)),
                ),
                (
                    "shiftLong",
                    "(JI)J",
                    &[Long(JLong(-1)), Int(JInt(65))],
                    Long(JLong(i64::MAX)),
                ),
                (
                    "mixedWide",
                    "(IJD)J",
                    &[Int(JInt(3)), Long(JLong(1 << 40)), Double(JDouble(-2.75))],
                    Long(JLong(1_099_511_627_777)),
                ),
            ];
            for &(name, desc, args, expected) in cases {
                for slice in [1, 10_000] {
                    assert_value(run_method(owner, name, desc, args, slice), expected);
                }
            }
        },
    );
}

#[test]
fn java_branches_loops_and_switches_survive_yielding() {
    isolated(
        concat!(
            module_path!(),
            "::java_branches_loops_and_switches_survive_yielding"
        ),
        || {
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let owner = load_normal_klass(&cld, &SymbolTable::intern("Cases")).unwrap();
            for slice in [1, 3, 10_000] {
                for (n, expected) in [(0, 0), (1, 1), (100, 5050)] {
                    assert_value(
                        run_method(owner, "sum", "(I)I", &[Int(JInt(n))], slice),
                        Int(JInt(expected)),
                    );
                }
                for (a, b, expected) in [(1, 2, -1), (2, 2, 0), (3, 2, 1)] {
                    assert_value(
                        run_method(
                            owner,
                            "compare",
                            "(II)I",
                            &[Int(JInt(a)), Int(JInt(b))],
                            slice,
                        ),
                        Int(JInt(expected)),
                    );
                }
                for (key, expected) in [(-2, 44), (-1, 11), (0, 22), (1, 33), (2, 44)] {
                    assert_value(
                        run_method(owner, "denseSwitch", "(I)I", &[Int(JInt(key))], slice),
                        Int(JInt(expected)),
                    );
                }
                for (key, expected) in [
                    (-1001, 8),
                    (-1000, 5),
                    (0, 8),
                    (7, 6),
                    (1_000_000, 7),
                    (i32::MAX, 8),
                ] {
                    assert_value(
                        run_method(owner, "sparseSwitch", "(I)I", &[Int(JInt(key))], slice),
                        Int(JInt(expected)),
                    );
                }
            }
        },
    );
}

#[test]
fn java_conversions_floats_and_returns() {
    isolated(
        concat!(module_path!(), "::java_conversions_floats_and_returns"),
        || {
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let owner = load_normal_klass(&cld, &SymbolTable::intern("Cases")).unwrap();
            for (input, expected) in [
                (f64::NAN, 0),
                (f64::INFINITY, i32::MAX),
                (f64::NEG_INFINITY, i32::MIN),
                (-3.75, -3),
            ] {
                assert_value(
                    run_method(owner, "toInt", "(D)I", &[Double(JDouble(input))], 1),
                    Int(JInt(expected)),
                );
            }
            assert_value(
                run_method(owner, "toByte", "(I)I", &[Int(JInt(255))], 1),
                Int(JInt(-1)),
            );
            assert_value(
                run_method(owner, "toChar", "(I)I", &[Int(JInt(-1))], 1),
                Int(JInt(65535)),
            );
            assert_value(
                run_method(
                    owner,
                    "divideFloat",
                    "(FF)F",
                    &[Float(JFloat(1.0)), Float(JFloat(0.0))],
                    1,
                ),
                Float(JFloat(f32::INFINITY)),
            );
            assert_value(
                run_method(
                    owner,
                    "divideFloat",
                    "(FF)F",
                    &[Float(JFloat(0.0)), Float(JFloat(0.0))],
                    1,
                ),
                Float(JFloat(f32::NAN)),
            );
            assert_value(
                run_method(owner, "negateDouble", "(D)D", &[Double(JDouble(0.0))], 1),
                Double(JDouble(-0.0)),
            );
            let null = Ref(NObjPtr::null());
            assert_value(
                run_method(
                    owner,
                    "reference",
                    "(Ljava/lang/Object;)Ljava/lang/Object;",
                    &[null],
                    1,
                ),
                null,
            );
            assert_value(
                run_method(owner, "isNull", "(Ljava/lang/Object;)I", &[null], 1),
                Int(JInt(1)),
            );
            assert!(matches!(
                run_method(owner, "nothing", "()V", &[], 1),
                MethodReturn::Void
            ));
            for (name, desc, expected) in [
                ("constantInt", "()I", Int(JInt(123456789))),
                ("constantLong", "()J", Long(JLong(-81985529216486895))),
                ("constantFloat", "()F", Float(JFloat(1.25))),
                ("constantDouble", "()D", Double(JDouble(3.25))),
            ] {
                assert_value(run_method(owner, name, desc, &[], 1), expected);
            }
        },
    );
}

#[test]
fn java_division_error_preserves_invocation_context() {
    isolated(
        concat!(
            module_path!(),
            "::java_division_error_preserves_invocation_context"
        ),
        || {
            init_vm(fixture_dir());
            let cld = ClassLoaderData::new(None);
            let owner = load_normal_klass(&cld, &SymbolTable::intern("Cases")).unwrap();
            let call = Invocation::lookup_declared(
                owner,
                "divide",
                "(II)I",
                &[Int(JInt(7)), Int(JInt(0))],
            )
            .unwrap();
            let mut thread = JavaThread::new();
            ExecDispatcher::start(&mut thread, call).unwrap();
            let error = match ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(100)) {
                Err(error) => error,
                Ok(_) => panic!("division by zero did not return an execution error"),
            };
            assert!(matches!(error.kind, ExecErrorKind::DivisionByZero));
            let brief = error.brief.unwrap();
            assert_eq!(brief.klass, "Cases");
            assert_eq!(brief.method, "divide");
            assert_eq!(brief.desc, "(II)I");
            assert!(brief.instruction_index.is_some());
        },
    );
}
