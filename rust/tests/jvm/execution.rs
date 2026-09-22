use klover::engines::interpreter::{
    interpreter::Interpreter, interpreter_frame::InterpreterFrame, slot::Slot,
    step_outcome::StepControl,
};
use klover::{
    class_loader::bs_cld::BootstrapCLD,
    code::instructions::{InstIdx, Instruction, Instruction::*, LocalIdx},
    engines::{
        exec_dispatcher::{DispatchOutcome, EngineExit, ExecBudget, ExecDispatcher, MethodReturn},
        exec_error::ExecErrorKind,
        invocation::Invocation,
    },
    gc_bindings::oop_hierarchy::NObjPtr,
    oops::jvalue::JValue,
    runtime::java_thread::JavaThread,
};
use std::sync::Mutex;

fn invocation(class: &str, method: &str, desc: &str, args: &[JValue]) -> Invocation {
    crate::support::init_vm();
    // Bootstrap class definition is not atomic yet. Serialize test fixture loads.
    static LOADING: Mutex<()> = Mutex::new(());
    let _guard = LOADING.lock().unwrap();
    let owner = BootstrapCLD::find_class(class)
        .unwrap()
        .as_normal_klass_ref()
        .unwrap();
    Invocation::try_new(owner, method, desc, args).unwrap()
}

fn frame() -> InterpreterFrame {
    InterpreterFrame::new(invocation(
        "InstructionOps",
        "scratch",
        "(IIIIII)I",
        &[JValue::Int(0); 6],
    ))
}

fn step(f: &mut InterpreterFrame, inst: Instruction) {
    assert!(matches!(
        Interpreter::execute_instruction(f, inst).unwrap(),
        StepControl::Continue
    ));
}

fn push(f: &mut InterpreterFrame, value: JValue) {
    match value {
        JValue::Int(v) => f.push_int(v),
        JValue::Long(v) => f.push_long(v),
        JValue::Float(v) => f.push_float(v),
        JValue::Double(v) => f.push_double(v),
        JValue::Ref(v) => f.push_ref(v),
        _ => panic!("use computational types in instruction tests"),
    }
}

fn assert_value(actual: JValue, expected: JValue) {
    match (actual, expected) {
        (JValue::Int(a), JValue::Int(b)) => assert_eq!(a, b),
        (JValue::Long(a), JValue::Long(b)) => assert_eq!(a, b),
        (JValue::Float(a), JValue::Float(b)) if b.is_nan() => assert!(a.is_nan()),
        (JValue::Double(a), JValue::Double(b)) if b.is_nan() => assert!(a.is_nan()),
        (JValue::Float(a), JValue::Float(b)) => assert_eq!(a.to_bits(), b.to_bits()),
        (JValue::Double(a), JValue::Double(b)) => assert_eq!(a.to_bits(), b.to_bits()),
        (JValue::Ref(a), JValue::Ref(b)) => assert_eq!(a.as_u32(), b.as_u32()),
        other => panic!("value mismatch: {other:?}"),
    }
}

fn assert_top(f: &mut InterpreterFrame, expected: JValue) {
    let actual = match expected {
        JValue::Int(_) => JValue::Int(f.pop_int().unwrap()),
        JValue::Long(_) => JValue::Long(f.pop_long().unwrap()),
        JValue::Float(_) => JValue::Float(f.pop_float().unwrap()),
        JValue::Double(_) => JValue::Double(f.pop_double().unwrap()),
        JValue::Ref(_) => JValue::Ref(f.pop_ref().unwrap()),
        _ => unreachable!(),
    };
    assert_value(actual, expected);
}

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
    let mut f = frame();
    step(&mut f, NOp);
    assert!(f.operand_stack().is_empty());
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
fn stack_instruction_forms_and_invalid_categories() {
    use JValue::*;
    // Each vector is ordered bottom-to-top, with long/double treated as whole values.
    let cases = vec![
        (Pop, vec![Int(1)], vec![]),
        (Pop2, vec![Int(1), Int(2)], vec![]),
        (Pop2, vec![Long(1)], vec![]),
        (Dup, vec![Int(1)], vec![Int(1), Int(1)]),
        (DupX1, vec![Int(1), Int(2)], vec![Int(2), Int(1), Int(2)]),
        (
            DupX2,
            vec![Int(1), Int(2), Int(3)],
            vec![Int(3), Int(1), Int(2), Int(3)],
        ),
        (DupX2, vec![Long(1), Int(2)], vec![Int(2), Long(1), Int(2)]),
        (
            Dup2,
            vec![Int(1), Int(2)],
            vec![Int(1), Int(2), Int(1), Int(2)],
        ),
        (Dup2, vec![Double(1.25)], vec![Double(1.25), Double(1.25)]),
        (
            Dup2X1,
            vec![Int(1), Int(2), Int(3)],
            vec![Int(2), Int(3), Int(1), Int(2), Int(3)],
        ),
        (
            Dup2X1,
            vec![Int(1), Long(2)],
            vec![Long(2), Int(1), Long(2)],
        ),
        (
            Dup2X2,
            vec![Int(1), Int(2), Int(3), Int(4)],
            vec![Int(3), Int(4), Int(1), Int(2), Int(3), Int(4)],
        ),
        (
            Dup2X2,
            vec![Long(1), Int(2), Int(3)],
            vec![Int(2), Int(3), Long(1), Int(2), Int(3)],
        ),
        (
            Dup2X2,
            vec![Int(1), Int(2), Long(3)],
            vec![Long(3), Int(1), Int(2), Long(3)],
        ),
        (
            Dup2X2,
            vec![Long(1), Double(2.25)],
            vec![Double(2.25), Long(1), Double(2.25)],
        ),
        (Swap, vec![Int(1), Float(2.5)], vec![Float(2.5), Int(1)]),
    ];
    for (inst, input, expected) in cases {
        let mut f = frame();
        f.push_int(99); // Every instruction must preserve the untouched stack prefix.
        for v in input {
            push(&mut f, v);
        }
        step(&mut f, inst);
        for v in expected.into_iter().rev() {
            assert_top(&mut f, v);
        }
        assert_top(&mut f, Int(99));
        assert!(f.operand_stack().is_empty());
    }
    for inst in [Pop, Dup, DupX1, DupX2, Swap] {
        let mut f = frame();
        f.push_long(5);
        assert!(
            matches!(Interpreter::execute_instruction(&mut f, inst), Err(e) if matches!(e.kind, ExecErrorKind::MismatchSlotType))
        );
        assert_top(&mut f, Long(5));
    }
    for inst in [Pop2, Dup2, Dup2X1, Dup2X2] {
        let mut f = frame();
        f.push_long(5);
        f.push_int(2); // Two top slots are half a long plus an int.
        assert!(
            matches!(Interpreter::execute_instruction(&mut f, inst), Err(e) if matches!(e.kind, ExecErrorKind::MismatchSlotType))
        );
    }
    let mut f = frame();
    assert!(
        matches!(Interpreter::execute_instruction(&mut f, Pop), Err(e) if matches!(e.kind, ExecErrorKind::StackUnderflow))
    );
}

#[test]
fn local_type_checks_and_overlapping_wide_stores() {
    use JValue::*;
    for (store, load, value) in [
        (IStore(LocalIdx(4)), ILoad(LocalIdx(4)), Int(-17)),
        (FStore(LocalIdx(4)), FLoad(LocalIdx(4)), Float(1.5)),
        (
            AStore(LocalIdx(4)),
            ALoad(LocalIdx(4)),
            Ref(NObjPtr::null()),
        ),
        (LStore(LocalIdx(4)), LLoad(LocalIdx(4)), Long(i64::MIN)),
        (DStore(LocalIdx(4)), DLoad(LocalIdx(4)), Double(-0.0)),
    ] {
        let mut f = frame();
        push(&mut f, value);
        step(&mut f, store);
        step(&mut f, load);
        assert_top(&mut f, value);
    }
    let mut f = frame();
    f.push_long(12);
    step(&mut f, LStore(LocalIdx(0)));
    f.push_int(3);
    step(&mut f, IStore(LocalIdx(1)));
    assert!(matches!(f.get_local(0).unwrap(), Slot::Unused));
    assert!(
        matches!(Interpreter::execute_instruction(&mut f, LLoad(LocalIdx(0))), Err(e) if matches!(e.kind, ExecErrorKind::MismatchSlotType))
    );
    f.push_double(1.5);
    step(&mut f, DStore(LocalIdx(2)));
    f.push_long(99);
    step(&mut f, LStore(LocalIdx(1)));
    assert!(matches!(f.get_local(3).unwrap(), Slot::Unused));
    step(&mut f, LLoad(LocalIdx(1)));
    assert_top(&mut f, Long(99));
    f.push_long(5);
    assert!(
        matches!(Interpreter::execute_instruction(&mut f, LStore(LocalIdx(5))), Err(e) if matches!(e.kind, ExecErrorKind::InvalidLocalIndex(6)))
    );
    assert!(matches!(f.get_local(5).unwrap(), Slot::Int(0)));
    assert!(
        matches!(Interpreter::execute_instruction(&mut f, FLoad(LocalIdx(5))), Err(e) if matches!(e.kind, ExecErrorKind::MismatchSlotType))
    );
    assert!(
        matches!(Interpreter::execute_instruction(&mut f, Goto(InstIdx(usize::MAX))), Err(e) if matches!(e.kind, ExecErrorKind::InvalidBranchTarget(_)))
    );
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
fn arithmetic_boundary_cases() {
    use JValue::*;
    let cases = vec![
        (IAdd, vec![Int(i32::MAX), Int(1)], Int(i32::MIN)),
        (ISub, vec![Int(i32::MIN), Int(1)], Int(i32::MAX)),
        (IMul, vec![Int(i32::MAX), Int(2)], Int(-2)),
        (IDiv, vec![Int(i32::MIN), Int(-1)], Int(i32::MIN)),
        (IRem, vec![Int(i32::MIN), Int(-1)], Int(0)),
        (INeg, vec![Int(i32::MIN)], Int(i32::MIN)),
        (IShL, vec![Int(1), Int(-1)], Int(i32::MIN)),
        (IShL, vec![Int(7), Int(32)], Int(7)),
        (IShR, vec![Int(-8), Int(33)], Int(-4)),
        (IUShR, vec![Int(-1), Int(-1)], Int(1)),
        (LAdd, vec![Long(i64::MAX), Long(1)], Long(i64::MIN)),
        (LSub, vec![Long(i64::MIN), Long(1)], Long(i64::MAX)),
        (LMul, vec![Long(i64::MAX), Long(2)], Long(-2)),
        (LDiv, vec![Long(i64::MIN), Long(-1)], Long(i64::MIN)),
        (LRem, vec![Long(i64::MIN), Long(-1)], Long(0)),
        (LNeg, vec![Long(i64::MIN)], Long(i64::MIN)),
        (LShL, vec![Long(1), Int(-1)], Long(i64::MIN)),
        (LShL, vec![Long(7), Int(64)], Long(7)),
        (LShR, vec![Long(-8), Int(65)], Long(-4)),
        (LUShR, vec![Long(-1), Int(-1)], Long(1)),
        (FDiv, vec![Float(1.0), Float(0.0)], Float(f32::INFINITY)),
        (FDiv, vec![Float(0.0), Float(0.0)], Float(f32::NAN)),
        (FRem, vec![Float(-4.0), Float(2.0)], Float(-0.0)),
        (
            FRem,
            vec![Float(f32::INFINITY), Float(2.0)],
            Float(f32::NAN),
        ),
        (FRem, vec![Float(2.0), Float(f32::INFINITY)], Float(2.0)),
        (FNeg, vec![Float(0.0)], Float(-0.0)),
        (DDiv, vec![Double(1.0), Double(0.0)], Double(f64::INFINITY)),
        (DDiv, vec![Double(0.0), Double(0.0)], Double(f64::NAN)),
        (DRem, vec![Double(-4.0), Double(2.0)], Double(-0.0)),
        (
            DRem,
            vec![Double(f64::INFINITY), Double(2.0)],
            Double(f64::NAN),
        ),
        (DRem, vec![Double(2.0), Double(f64::INFINITY)], Double(2.0)),
        (DNeg, vec![Double(0.0)], Double(-0.0)),
    ];
    for (inst, args, expected) in cases {
        let mut f = frame();
        for value in args {
            push(&mut f, value);
        }
        step(&mut f, inst);
        assert_top(&mut f, expected);
        assert!(f.operand_stack().is_empty());
    }
    for (inst, args) in [
        (IDiv, vec![Int(1), Int(0)]),
        (IRem, vec![Int(1), Int(0)]),
        (LDiv, vec![Long(1), Long(0)]),
        (LRem, vec![Long(1), Long(0)]),
    ] {
        let mut f = frame();
        for value in args {
            push(&mut f, value);
        }
        assert!(
            matches!(Interpreter::execute_instruction(&mut f, inst), Err(e) if matches!(e.kind, ExecErrorKind::DivisionByZero))
        );
    }
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
    let cases = vec![
        (F2I, Float(f32::NAN), Int(0)),
        (F2I, Float(f32::INFINITY), Int(i32::MAX)),
        (F2I, Float(f32::NEG_INFINITY), Int(i32::MIN)),
        (F2L, Float(f32::NAN), Long(0)),
        (F2L, Float(f32::INFINITY), Long(i64::MAX)),
        (F2L, Float(f32::NEG_INFINITY), Long(i64::MIN)),
        (D2I, Double(f64::NAN), Int(0)),
        (D2I, Double(f64::INFINITY), Int(i32::MAX)),
        (D2I, Double(f64::NEG_INFINITY), Int(i32::MIN)),
        (D2L, Double(f64::NAN), Long(0)),
        (D2L, Double(f64::INFINITY), Long(i64::MAX)),
        (D2L, Double(f64::NEG_INFINITY), Long(i64::MIN)),
        (L2I, Long(0x1_ffff_ffff), Int(-1)),
        (I2F, Int(16_777_217), Float(16_777_216.0)),
        (
            L2D,
            Long(9_007_199_254_740_993),
            Double(9_007_199_254_740_992.0),
        ),
        (D2F, Double(f64::MAX), Float(f32::INFINITY)),
        (D2F, Double(-0.0), Float(-0.0)),
    ];
    for (inst, input, expected) in cases {
        let mut f = frame();
        push(&mut f, input);
        step(&mut f, inst);
        assert_top(&mut f, expected);
    }
    for (inst, left, right, expected) in [
        (LCmp, Long(i64::MIN), Long(i64::MAX), -1),
        (LCmp, Long(i64::MAX), Long(i64::MIN), 1),
        (LCmp, Long(5), Long(5), 0),
        (FCmpL, Float(f32::NAN), Float(1.0), -1),
        (FCmpG, Float(1.0), Float(f32::NAN), 1),
        (DCmpL, Double(1.0), Double(f64::NAN), -1),
        (DCmpG, Double(f64::NAN), Double(1.0), 1),
        (FCmpL, Float(-0.0), Float(0.0), 0),
        (DCmpG, Double(f64::INFINITY), Double(1.0), 1),
    ] {
        let mut f = frame();
        push(&mut f, left);
        push(&mut f, right);
        step(&mut f, inst);
        assert_top(&mut f, Int(expected));
    }
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
