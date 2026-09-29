//! End-to-end execution of methods loaded from Java class files.
use crate::support::{assert_value, invocation};
use klover::engines::interpreter::{interpreter::Interpreter, interpreter_frame::InterpreterFrame};
use klover::{
    engines::{
        exec_dispatcher::{DispatchOutcome, EngineExit, ExecBudget, ExecDispatcher, MethodReturn},
        exec_error::ExecErrorKind,
    },
    gc_bindings::oop_hierarchy::NObjPtr,
    oops::jvalue::JValue,
    runtime::java_thread::JavaThread,
};

fn run(class: &str, method: &str, desc: &str, args: &[JValue], fuel: u64) -> MethodReturn {
    let mut thread = JavaThread::new();
    ExecDispatcher::start(&mut thread, invocation(class, method, desc, args)).unwrap();
    assert!(matches!(
        ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(0)).unwrap(),
        DispatchOutcome::Yielded
    ));
    for _ in 0..10000 {
        match ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(fuel))
            .unwrap_or_else(|error| panic!("{class}.{method}{desc}: {error:?}"))
        {
            DispatchOutcome::Yielded => {}
            DispatchOutcome::Completed(value) => return value,
        }
    }
    panic!("fixture did not terminate: {class}.{method}");
}

fn check(class: &str, method: &str, desc: &str, args: &[JValue], expected: JValue) {
    for fuel in [1, 10000] {
        let MethodReturn::Value(value) = run(class, method, desc, args, fuel) else {
            panic!("expected value")
        };
        assert_value(value, expected);
    }
}

#[test]
fn simple_addition_through_interpreter() {
    let mut frame = InterpreterFrame::new(invocation(
        "SimpleAddition",
        "add",
        "(II)I",
        &[JValue::Int(1), JValue::Int(2)],
    ));
    let exit = Interpreter::execute(&mut frame, &mut ExecBudget::new(5)).unwrap();
    assert!(matches!(
        exit,
        EngineExit::Return(MethodReturn::Value(JValue::Int(3)))
    ));
}

#[test]
fn simple_addition_through_dispatcher() {
    let mut thread = JavaThread::new();
    ExecDispatcher::start(
        &mut thread,
        invocation(
            "SimpleAddition",
            "add",
            "(II)I",
            &[JValue::Int(1), JValue::Int(2)],
        ),
    )
    .unwrap();
    assert!(matches!(
        ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(5)).unwrap(),
        DispatchOutcome::Completed(MethodReturn::Value(JValue::Int(3)))
    ));
}

#[test]
fn constants_and_java_local_round_trips() {
    use JValue::*;
    for (method, expected) in [
        ("intMinusOne", -1),
        ("intZero", 0),
        ("intOne", 1),
        ("intTwo", 2),
        ("intThree", 3),
        ("intFour", 4),
        ("intFive", 5),
        ("byteImmediate", -100),
        ("shortImmediate", -30000),
    ] {
        check("ConstantOps", method, "()I", &[], Int(expected));
    }
    for (method, desc, expected) in [
        ("longZero", "()J", Long(0)),
        ("longOne", "()J", Long(1)),
        ("floatZero", "()F", Float(0.0)),
        ("floatOne", "()F", Float(1.0)),
        ("floatTwo", "()F", Float(2.0)),
        ("doubleZero", "()D", Double(0.0)),
        ("doubleOne", "()D", Double(1.0)),
        ("nullConstant", "()Ljava/lang/Object;", Ref(NObjPtr::null())),
    ] {
        check("ConstantOps", method, desc, &[], expected);
    }
    for (method, desc, value) in [
        ("longStore", "(J)J", Long(i64::MIN + 9)),
        ("floatStore", "(F)F", Float(-0.0)),
        ("doubleStore", "(D)D", Double(-123.25)),
    ] {
        check("InstructionOps", method, desc, &[value], value);
    }
    check(
        "StoreOps",
        "intIndexed",
        "(IIII)I",
        &[Int(1), Int(2), Int(3), Int(9)],
        Int(10),
    );
    check(
        "ReferenceLoads",
        "roundTrip",
        "(Ljava/lang/Object;)Ljava/lang/Object;",
        &[Ref(NObjPtr::null())],
        Ref(NObjPtr::null()),
    );
    assert!(matches!(
        run("ControlFlow", "doNothing", "()V", &[], 1),
        MethodReturn::Void
    ));
}

#[test]
fn loops_branches_and_budget_resumption() {
    use JValue::*;
    for (method, input, result) in [
        ("sum", 0, 0),
        ("sum", 1, 1),
        ("sum", 10, 55),
        ("countTo", 10, 10),
        ("factorial", 5, 120),
        ("firstEven", 3, 4),
    ] {
        check("ControlFlow", method, "(I)I", &[Int(input)], Int(result));
    }
    for (a, b) in [(2, 7), (7, 2), (3, 3), (i32::MIN, i32::MAX)] {
        check(
            "ControlFlow",
            "max",
            "(II)I",
            &[Int(a), Int(b)],
            Int(a.max(b)),
        );
        for (method, expected) in [
            ("intsEqual", a == b),
            ("intsNotEqual", a != b),
            ("intLessThan", a < b),
            ("intGreaterOrEqual", a >= b),
            ("intGreaterThan", a > b),
            ("intLessOrEqual", a <= b),
        ] {
            check(
                "ControlFlow",
                method,
                "(II)Z",
                &[Int(a), Int(b)],
                Int(expected as i32),
            );
        }
    }
    for a in [-1, 0, 1] {
        for (method, expected) in [
            ("isZero", a == 0),
            ("isNonZero", a != 0),
            ("isNegative", a < 0),
            ("isNonNegative", a >= 0),
            ("isPositive", a > 0),
            ("isNonPositive", a <= 0),
        ] {
            check(
                "ControlFlow",
                method,
                "(I)Z",
                &[Int(a)],
                Int(expected as i32),
            );
        }
    }
    check(
        "InstructionOps",
        "wideIncrement",
        "(I)I",
        &[Int(i32::MAX)],
        Int(i32::MIN + 999),
    );
    check(
        "InstructionOps",
        "negativeIncrement",
        "(I)I",
        &[Int(3)],
        Int(-997),
    );
    for (name, expected) in [("isNull", 1), ("isNonNull", 0)] {
        check(
            "InstructionOps",
            name,
            "(Ljava/lang/Object;)Z",
            &[Ref(NObjPtr::null())],
            Int(expected),
        );
    }
    for (name, expected) in [("same", 1), ("different", 0)] {
        check(
            "InstructionOps",
            name,
            "(Ljava/lang/Object;Ljava/lang/Object;)Z",
            &[Ref(NObjPtr::null()); 2],
            Int(expected),
        );
    }
}

#[test]
fn arithmetic_java_fixtures() {
    use JValue::*;
    check(
        "ArithmeticOps",
        "iadd",
        "(II)I",
        &[Int(-7), Int(3)],
        Int(-4),
    );
    check(
        "ArithmeticOps",
        "isub",
        "(II)I",
        &[Int(-7), Int(3)],
        Int(-10),
    );
    check(
        "ArithmeticOps",
        "imul",
        "(II)I",
        &[Int(-7), Int(3)],
        Int(-21),
    );
    check(
        "ArithmeticOps",
        "idiv",
        "(II)I",
        &[Int(-7), Int(3)],
        Int(-2),
    );
    check(
        "ArithmeticOps",
        "irem",
        "(II)I",
        &[Int(-7), Int(3)],
        Int(-1),
    );
    check("ArithmeticOps", "ineg", "(I)I", &[Int(-7)], Int(7));
    check("ArithmeticOps", "iand", "(II)I", &[Int(6), Int(3)], Int(2));
    check("ArithmeticOps", "ior", "(II)I", &[Int(6), Int(3)], Int(7));
    check("ArithmeticOps", "ixor", "(II)I", &[Int(6), Int(3)], Int(5));
    check("ArithmeticOps", "ishl", "(II)I", &[Int(3), Int(2)], Int(12));
    check(
        "ArithmeticOps",
        "ishr",
        "(II)I",
        &[Int(-12), Int(2)],
        Int(-3),
    );
    check(
        "ArithmeticOps",
        "iushr",
        "(II)I",
        &[Int(-1), Int(1)],
        Int(i32::MAX),
    );
    check(
        "ArithmeticOps",
        "ladd",
        "(JJ)J",
        &[Long(-7), Long(3)],
        Long(-4),
    );
    check(
        "ArithmeticOps",
        "lsub",
        "(JJ)J",
        &[Long(-7), Long(3)],
        Long(-10),
    );
    check(
        "ArithmeticOps",
        "lmul",
        "(JJ)J",
        &[Long(-7), Long(3)],
        Long(-21),
    );
    check(
        "ArithmeticOps",
        "ldiv",
        "(JJ)J",
        &[Long(-7), Long(3)],
        Long(-2),
    );
    check(
        "ArithmeticOps",
        "lrem",
        "(JJ)J",
        &[Long(-7), Long(3)],
        Long(-1),
    );
    check("ArithmeticOps", "lneg", "(J)J", &[Long(-7)], Long(7));
    check(
        "ArithmeticOps",
        "land",
        "(JJ)J",
        &[Long(6), Long(3)],
        Long(2),
    );
    check(
        "ArithmeticOps",
        "lor",
        "(JJ)J",
        &[Long(6), Long(3)],
        Long(7),
    );
    check(
        "ArithmeticOps",
        "lxor",
        "(JJ)J",
        &[Long(6), Long(3)],
        Long(5),
    );
    check(
        "ArithmeticOps",
        "lshl",
        "(JI)J",
        &[Long(3), Int(2)],
        Long(12),
    );
    check(
        "ArithmeticOps",
        "lshr",
        "(JI)J",
        &[Long(-12), Int(2)],
        Long(-3),
    );
    check(
        "ArithmeticOps",
        "lushr",
        "(JI)J",
        &[Long(-1), Int(1)],
        Long(i64::MAX),
    );
    check(
        "ArithmeticOps",
        "fadd",
        "(FF)F",
        &[Float(-7.0), Float(3.0)],
        Float(-4.0),
    );
    check(
        "ArithmeticOps",
        "fsub",
        "(FF)F",
        &[Float(-7.0), Float(3.0)],
        Float(-10.0),
    );
    check(
        "ArithmeticOps",
        "fmul",
        "(FF)F",
        &[Float(-7.0), Float(3.0)],
        Float(-21.0),
    );
    check(
        "ArithmeticOps",
        "fdiv",
        "(FF)F",
        &[Float(-7.5), Float(3.0)],
        Float(-2.5),
    );
    check(
        "ArithmeticOps",
        "frem",
        "(FF)F",
        &[Float(-7.5), Float(3.0)],
        Float(-1.5),
    );
    check("ArithmeticOps", "fneg", "(F)F", &[Float(-7.0)], Float(7.0));
    check(
        "ArithmeticOps",
        "dadd",
        "(DD)D",
        &[Double(-7.0), Double(3.0)],
        Double(-4.0),
    );
    check(
        "ArithmeticOps",
        "dsub",
        "(DD)D",
        &[Double(-7.0), Double(3.0)],
        Double(-10.0),
    );
    check(
        "ArithmeticOps",
        "dmul",
        "(DD)D",
        &[Double(-7.0), Double(3.0)],
        Double(-21.0),
    );
    check(
        "ArithmeticOps",
        "ddiv",
        "(DD)D",
        &[Double(-7.5), Double(3.0)],
        Double(-2.5),
    );
    check(
        "ArithmeticOps",
        "drem",
        "(DD)D",
        &[Double(-7.5), Double(3.0)],
        Double(-1.5),
    );
    check(
        "ArithmeticOps",
        "dneg",
        "(D)D",
        &[Double(-7.0)],
        Double(7.0),
    );
}

#[test]
fn conversions_and_nan_comparisons() {
    use JValue::*;
    check("Wide", "i2l", "(I)J", &[Int(-17)], Long(-17));
    check("Wide", "i2f", "(I)F", &[Int(-17)], Float(-17.0));
    check("Wide", "i2d", "(I)D", &[Int(-17)], Double(-17.0));
    check("Wide", "l2i", "(J)I", &[Long(-17)], Int(-17));
    check("Wide", "l2f", "(J)F", &[Long(-17)], Float(-17.0));
    check("Wide", "l2d", "(J)D", &[Long(-17)], Double(-17.0));
    check("Wide", "f2i", "(F)I", &[Float(-17.75)], Int(-17));
    check("Wide", "f2l", "(F)J", &[Float(-17.75)], Long(-17));
    check("Wide", "f2d", "(F)D", &[Float(-17.75)], Double(-17.75));
    check("Wide", "d2i", "(D)I", &[Double(-17.75)], Int(-17));
    check("Wide", "d2l", "(D)J", &[Double(-17.75)], Long(-17));
    check("Wide", "d2f", "(D)F", &[Double(-17.75)], Float(-17.75));
    check("InstructionOps", "narrowByte", "(I)I", &[Int(255)], Int(-1));
    check(
        "InstructionOps",
        "narrowChar",
        "(I)I",
        &[Int(-1)],
        Int(65535),
    );
    check(
        "InstructionOps",
        "narrowShort",
        "(I)I",
        &[Int(65535)],
        Int(-1),
    );
    check(
        "Wide",
        "fbranch",
        "(FF)I",
        &[Float(f32::NAN), Float(1.0)],
        Int(0),
    );
    check(
        "Wide",
        "fbranch",
        "(FF)I",
        &[Float(1.0), Float(2.0)],
        Int(-1),
    );
    check(
        "Wide",
        "dbranch",
        "(DD)I",
        &[Double(3.0), Double(2.0)],
        Int(1),
    );
    check(
        "Wide",
        "dbranch",
        "(DD)I",
        &[Double(f64::NAN), Double(2.0)],
        Int(0),
    );
    check(
        "InstructionOps",
        "longCompare",
        "(JJ)I",
        &[Long(i64::MIN), Long(i64::MAX)],
        Int(-1),
    );
}

mod algorithms;
