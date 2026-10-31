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
        jvalue::JValue::{self, *},
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
            let cld = ClassLoaderData::new();
            let owner = load_normal_klass(&cld, &SymbolTable::intern("Cases")).unwrap();
            let cases: &[(&str, &str, &[JValue], JValue)] = &[
                ("subtract", "(II)I", &[Int(7), Int(12)], Int(-5)),
                ("add", "(II)I", &[Int(i32::MAX), Int(1)], Int(i32::MIN)),
                (
                    "multiply",
                    "(JJ)J",
                    &[Long(1 << 40), Long(-3)],
                    Long(-3_298_534_883_328),
                ),
                ("divide", "(II)I", &[Int(i32::MIN), Int(-1)], Int(i32::MIN)),
                ("divide", "(II)I", &[Int(-7), Int(3)], Int(-2)),
                ("remainder", "(II)I", &[Int(-7), Int(3)], Int(-1)),
                ("shift", "(II)I", &[Int(-1), Int(33)], Int(i32::MAX)),
                ("shiftLong", "(JI)J", &[Long(-1), Int(65)], Long(i64::MAX)),
                (
                    "mixedWide",
                    "(IJD)J",
                    &[Int(3), Long(1 << 40), Double(-2.75)],
                    Long(1_099_511_627_777),
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
            let cld = ClassLoaderData::new();
            let owner = load_normal_klass(&cld, &SymbolTable::intern("Cases")).unwrap();
            for slice in [1, 3, 10_000] {
                for (n, expected) in [(0, 0), (1, 1), (100, 5050)] {
                    assert_value(
                        run_method(owner, "sum", "(I)I", &[Int(n)], slice),
                        Int(expected),
                    );
                }
                for (a, b, expected) in [(1, 2, -1), (2, 2, 0), (3, 2, 1)] {
                    assert_value(
                        run_method(owner, "compare", "(II)I", &[Int(a), Int(b)], slice),
                        Int(expected),
                    );
                }
                for (key, expected) in [(-2, 44), (-1, 11), (0, 22), (1, 33), (2, 44)] {
                    assert_value(
                        run_method(owner, "denseSwitch", "(I)I", &[Int(key)], slice),
                        Int(expected),
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
                        run_method(owner, "sparseSwitch", "(I)I", &[Int(key)], slice),
                        Int(expected),
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
            let cld = ClassLoaderData::new();
            let owner = load_normal_klass(&cld, &SymbolTable::intern("Cases")).unwrap();
            for (input, expected) in [
                (f64::NAN, 0),
                (f64::INFINITY, i32::MAX),
                (f64::NEG_INFINITY, i32::MIN),
                (-3.75, -3),
            ] {
                assert_value(
                    run_method(owner, "toInt", "(D)I", &[Double(input)], 1),
                    Int(expected),
                );
            }
            assert_value(run_method(owner, "toByte", "(I)I", &[Int(255)], 1), Int(-1));
            assert_value(
                run_method(owner, "toChar", "(I)I", &[Int(-1)], 1),
                Int(65535),
            );
            assert_value(
                run_method(owner, "divideFloat", "(FF)F", &[Float(1.0), Float(0.0)], 1),
                Float(f32::INFINITY),
            );
            assert_value(
                run_method(owner, "divideFloat", "(FF)F", &[Float(0.0), Float(0.0)], 1),
                Float(f32::NAN),
            );
            assert_value(
                run_method(owner, "negateDouble", "(D)D", &[Double(0.0)], 1),
                Double(-0.0),
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
                Int(1),
            );
            assert!(matches!(
                run_method(owner, "nothing", "()V", &[], 1),
                MethodReturn::Void
            ));
            for (name, desc, expected) in [
                ("constantInt", "()I", Int(123456789)),
                ("constantLong", "()J", Long(-81985529216486895)),
                ("constantFloat", "()F", Float(1.25)),
                ("constantDouble", "()D", Double(3.25)),
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
            let cld = ClassLoaderData::new();
            let owner = load_normal_klass(&cld, &SymbolTable::intern("Cases")).unwrap();
            let call =
                Invocation::lookup_declared(owner, "divide", "(II)I", &[Int(7), Int(0)]).unwrap();
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
