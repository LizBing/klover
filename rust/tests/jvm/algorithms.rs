use super::*;
use JValue::*;

#[test]
fn remaining_integer_and_static_entry_methods() {
    for (method, args, desc, result) in [
        ("mul2", vec![Int(21)], "(I)I", 42),
        ("mul2", vec![Int(i32::MAX)], "(I)I", -2),
        ("neg", vec![Int(i32::MIN)], "(I)I", i32::MIN),
        ("sub", vec![Int(4), Int(9)], "(II)I", -5),
        ("rem", vec![Int(-17), Int(5)], "(II)I", -2),
        ("addConst", vec![], "()I", 15),
        ("pushByte", vec![], "()I", 100),
        ("pushShort", vec![], "()I", 10000),
    ] {
        check("Arith", method, desc, &args, Int(result));
    }
    check(
        "ArithmeticOps",
        "iinc",
        "(I)I",
        &[Int(i32::MAX)],
        Int(i32::MIN + 6),
    );
    check(
        "ControlFlow",
        "indexedStore",
        "(IIIII)I",
        &[Int(7), Int(0), Int(0), Int(0), Int(9)],
        Int(16),
    );
    check(
        "InstructionOps",
        "scratch",
        "(IIIIII)I",
        &[Int(42); 6],
        Int(42),
    );
    check("StaticCallee", "twice", "(I)I", &[Int(21)], Int(42));
    check(
        "StaticCallee",
        "addLong",
        "(JJ)J",
        &[Long(i64::MAX), Long(1)],
        Long(i64::MIN),
    );
    for fuel in [1, 10000] {
        assert!(matches!(
            run("StaticCallee", "consume", "(I)V", &[Int(42)], fuel),
            MethodReturn::Void
        ));
    }
}

#[test]
fn every_supported_store_and_reference_load_method() {
    for (method, desc, args, expected) in [
        ("int0", "()I", vec![], 10),
        ("int1", "(I)I", vec![Int(7)], 8),
        ("int2", "(II)I", vec![Int(7), Int(-9)], -2),
        ("int3", "(III)I", vec![Int(7), Int(-9), Int(5)], 3),
        (
            "intIndexed",
            "(IIII)I",
            vec![Int(7), Int(99), Int(99), Int(5)],
            12,
        ),
    ] {
        check("StoreOps", method, desc, &args, Int(expected));
    }
    for (index, suffix) in ["0", "1", "2", "3", "Indexed"].into_iter().enumerate() {
        let mut args = vec![Int(37); index];
        let desc = format!("({})V", "I".repeat(index));
        for fuel in [1, 10000] {
            assert!(matches!(
                run("StoreOps", &format!("ref{suffix}"), &desc, &args, fuel),
                MethodReturn::Void
            ));
        }
        args.push(Ref(NObjPtr::null()));
        let desc = format!(
            "({}Ljava/lang/Object;)Ljava/lang/Object;",
            "I".repeat(index)
        );
        check(
            "ReferenceLoads",
            &format!("load{suffix}"),
            &desc,
            &args,
            Ref(NObjPtr::null()),
        );
    }
}

#[test]
fn wide_arithmetic_and_bitwise_methods() {
    for (suffix, integer, float) in [
        ("add", -4, -4.0),
        ("sub", -10, -10.0),
        ("mul", -21, -21.0),
        ("div", -2, -7.0 / 3.0),
        ("rem", -1, -1.0),
    ] {
        check(
            "Wide",
            &format!("l{suffix}"),
            "(JJ)J",
            &[Long(-7), Long(3)],
            Long(integer),
        );
        check(
            "Wide",
            &format!("f{suffix}"),
            "(FF)F",
            &[Float(-7.0), Float(3.0)],
            Float(float as f32),
        );
        check(
            "Wide",
            &format!("d{suffix}"),
            "(DD)D",
            &[Double(-7.0), Double(3.0)],
            Double(float),
        );
    }
    check("Wide", "lneg", "(J)J", &[Long(i64::MIN)], Long(i64::MIN));
    check("Wide", "fneg", "(F)F", &[Float(0.0)], Float(-0.0));
    check("Wide", "dneg", "(D)D", &[Double(0.0)], Double(-0.0));
    for (suffix, expected) in [("and", 2), ("or", 7), ("xor", 5)] {
        check(
            "Wide",
            &format!("i{suffix}"),
            "(II)I",
            &[Int(6), Int(3)],
            Int(expected),
        );
        check(
            "Wide",
            &format!("l{suffix}"),
            "(JJ)J",
            &[Long(6), Long(3)],
            Long(expected.into()),
        );
    }
    for (method, left, shift, expected) in [
        ("ishl", 1, -1, i32::MIN),
        ("ishr", -8, 33, -4),
        ("iushr", -1, 31, 1),
    ] {
        check(
            "Wide",
            method,
            "(II)I",
            &[Int(left), Int(shift)],
            Int(expected),
        );
    }
    for (method, left, shift, expected) in [
        ("lshl", 1, -1, i64::MIN),
        ("lshr", -8, 65, -4),
        ("lushr", -1, 63, 1),
    ] {
        check(
            "Wide",
            method,
            "(JI)J",
            &[Long(left), Int(shift)],
            Long(expected),
        );
    }
}

#[test]
fn numeric_edges_through_real_class_files() {
    // These cover decoding, local argument placement, execution, and return together.
    for (method, args, expected) in [
        ("iadd", vec![Int(i32::MAX), Int(1)], Int(i32::MIN)),
        ("isub", vec![Int(i32::MIN), Int(1)], Int(i32::MAX)),
        ("idiv", vec![Int(i32::MIN), Int(-1)], Int(i32::MIN)),
        ("irem", vec![Int(i32::MIN), Int(-1)], Int(0)),
    ] {
        check("ArithmeticOps", method, "(II)I", &args, expected);
    }
    for (method, args, expected) in [
        ("ldiv", vec![Long(i64::MIN), Long(-1)], Long(i64::MIN)),
        ("lrem", vec![Long(i64::MIN), Long(-1)], Long(0)),
    ] {
        check("ArithmeticOps", method, "(JJ)J", &args, expected);
    }
    check(
        "ArithmeticOps",
        "fdiv",
        "(FF)F",
        &[Float(1.0), Float(0.0)],
        Float(f32::INFINITY),
    );
    check(
        "ArithmeticOps",
        "ddiv",
        "(DD)D",
        &[Double(0.0), Double(0.0)],
        Double(f64::NAN),
    );
    check(
        "ArithmeticOps",
        "frem",
        "(FF)F",
        &[Float(-4.0), Float(2.0)],
        Float(-0.0),
    );
    check(
        "ArithmeticOps",
        "drem",
        "(DD)D",
        &[Double(-4.0), Double(2.0)],
        Double(-0.0),
    );
    for (value, int, long) in [
        (f64::NAN, 0, 0),
        (f64::INFINITY, i32::MAX, i64::MAX),
        (f64::NEG_INFINITY, i32::MIN, i64::MIN),
        (-17.75, -17, -17),
    ] {
        check("Wide", "f2i", "(F)I", &[Float(value as f32)], Int(int));
        check("Wide", "d2i", "(D)I", &[Double(value)], Int(int));
        check("Wide", "f2l", "(F)J", &[Float(value as f32)], Long(long));
        check("Wide", "d2l", "(D)J", &[Double(value)], Long(long));
    }
    for (a, b, expected) in [
        (1.0, 2.0, -1),
        (2.0, 1.0, 1),
        (-0.0, 0.0, 0),
        (f64::NAN, 1.0, 0),
        (1.0, f64::NAN, 0),
    ] {
        check(
            "Wide",
            "fbranch",
            "(FF)I",
            &[Float(a as f32), Float(b as f32)],
            Int(expected),
        );
        check(
            "Wide",
            "dbranch",
            "(DD)I",
            &[Double(a), Double(b)],
            Int(expected),
        );
    }
    for (a, b, expected) in [
        (i64::MIN, i64::MAX, -1),
        (i64::MAX, i64::MIN, 1),
        (42, 42, 0),
    ] {
        check(
            "InstructionOps",
            "longCompare",
            "(JJ)I",
            &[Long(a), Long(b)],
            Int(expected),
        );
    }
}

#[test]
fn real_class_division_errors_reach_dispatcher() {
    for (method, desc, args) in [
        ("idiv", "(II)I", vec![Int(7), Int(0)]),
        ("irem", "(II)I", vec![Int(7), Int(0)]),
        ("ldiv", "(JJ)J", vec![Long(7), Long(0)]),
        ("lrem", "(JJ)J", vec![Long(7), Long(0)]),
    ] {
        for fuel in [1, 10000] {
            let mut thread = JavaThread::new();
            ExecDispatcher::start(
                &mut thread,
                invocation("ArithmeticOps", method, desc, &args),
            )
            .unwrap();
            let error = (0..20)
                .find_map(
                    |_| match ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(fuel)) {
                        Err(error) => Some(error),
                        Ok(DispatchOutcome::Yielded) => None,
                        Ok(DispatchOutcome::Completed(_)) => {
                            panic!("division by zero completed normally")
                        }
                    },
                )
                .expect("division by zero must terminate with an error");
            assert!(matches!(error.kind, ExecErrorKind::DivisionByZero));
            assert!(error.invocation.is_some());
        }
    }
}

#[test]
fn unsupported_constant_pool_loads_remain_explicit_errors() {
    for (method, desc) in [
        ("intPoolConstant", "()I"),
        ("floatPoolConstant", "()F"),
        ("longPoolConstant", "()J"),
        ("doublePoolConstant", "()D"),
    ] {
        let mut thread = JavaThread::new();
        ExecDispatcher::start(&mut thread, invocation("ConstantOps", method, desc, &[])).unwrap();
        assert!(
            matches!(ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(100)),
            Err(error) if matches!(error.kind, ExecErrorKind::UnsupportedInstruction))
        );
    }
}

#[test]
fn fibonacci_and_gcd_algorithms() {
    for (n, result) in [
        (0, 0),
        (1, 1),
        (2, 1),
        (10, 55),
        (30, 832040),
        (92, 7540113804746346429),
    ] {
        check("AlgorithmOps", "fibonacci", "(I)J", &[Int(n)], Long(result));
    }
    for (a, b, result) in [
        (0, 0, 0),
        (0, 42, 42),
        (42, 0, 42),
        (48, 18, 6),
        (1071, 462, 21),
        (17, 13, 1),
        (i32::MAX, 1, 1),
    ] {
        check(
            "AlgorithmOps",
            "gcd",
            "(II)I",
            &[Int(a), Int(b)],
            Int(result),
        );
    }
}

#[test]
fn prime_bitcount_and_nested_loop_algorithms() {
    for (n, result) in [
        (-7, 0),
        (0, 0),
        (1, 0),
        (2, 1),
        (3, 1),
        (4, 0),
        (49, 0),
        (97, 1),
        (997, 1),
        (999, 0),
    ] {
        check("AlgorithmOps", "isPrime", "(I)Z", &[Int(n)], Int(result));
    }
    for (value, count) in [
        (0, 0),
        (1, 1),
        (7, 3),
        (0x55555555, 16),
        (i32::MIN, 1),
        (i32::MAX, 31),
        (-1, 32),
    ] {
        check(
            "AlgorithmOps",
            "bitCount",
            "(I)I",
            &[Int(value)],
            Int(count),
        );
    }
    for (rows, columns, sum) in [(0, 5, 0), (5, 0, 0), (1, 1, 0), (2, 3, 9), (5, 7, 175)] {
        check(
            "AlgorithmOps",
            "nestedSum",
            "(II)I",
            &[Int(rows), Int(columns)],
            Int(sum),
        );
    }
}

#[test]
fn floating_point_polynomial_algorithm() {
    for (args, expected) in [
        ([Double(3.0), Double(2.0), Double(-4.0), Double(5.0)], 11.0),
        (
            [Double(-2.0), Double(0.5), Double(1.5), Double(-0.25)],
            -1.25,
        ),
        ([Double(0.0), Double(2.0), Double(-4.0), Double(5.0)], 5.0),
    ] {
        check(
            "AlgorithmOps",
            "polynomial",
            "(DDDD)D",
            &args,
            Double(expected),
        );
    }
}

#[test]
fn budget_yields_on_back_edges_and_zero_fuel_preserves_progress() {
    let mut spin = JavaThread::new();
    ExecDispatcher::start(&mut spin, invocation("AlgorithmOps", "spin", "()V", &[])).unwrap();
    for fuel in [0, 1, 2, 17, 0, 100] {
        let mut budget = ExecBudget::new(fuel);
        assert!(matches!(
            ExecDispatcher::poll(&mut spin, &mut budget).unwrap(),
            DispatchOutcome::Yielded
        ));
        assert_eq!(budget.remaining(), 0);
    }
    let mut thread = JavaThread::new();
    ExecDispatcher::start(
        &mut thread,
        invocation("AlgorithmOps", "fibonacci", "(I)J", &[Int(30)]),
    )
    .unwrap();
    for index in 0..10000 {
        assert!(matches!(
            ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(0)).unwrap(),
            DispatchOutcome::Yielded
        ));
        match ExecDispatcher::poll(&mut thread, &mut ExecBudget::new([1, 2, 3, 7][index % 4]))
            .unwrap()
        {
            DispatchOutcome::Yielded => {}
            DispatchOutcome::Completed(MethodReturn::Value(value)) => {
                assert_value(value, Long(832040));
                return;
            }
            DispatchOutcome::Completed(MethodReturn::Void) => panic!("missing fibonacci result"),
        }
    }
    panic!("zero-fuel polls must not prevent eventual completion");
}
