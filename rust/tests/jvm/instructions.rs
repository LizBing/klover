//! Instruction-level component checks. These remain integration tests because
//! constructing an InterpreterFrame currently requires a loaded method.
use crate::support::{assert_value, invocation};
use klover::{
    code::instructions::{InstIdx, Instruction, Instruction::*, LocalIdx},
    engines::{
        exec_error::ExecErrorKind,
        interpreter::{
            interpreter::Interpreter, interpreter_frame::InterpreterFrame, slot::Slot,
            step_outcome::StepControl,
        },
    },
    gc_bindings::oop_hierarchy::NObjPtr,
    oops::jvalue::JValue,
};

fn frame() -> InterpreterFrame<'static> {
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

#[test]
fn nop_preserves_empty_stack() {
    let mut f = frame();
    step(&mut f, NOp);
    assert!(f.operand_stack().is_empty());
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
fn conversion_edges_and_nan_comparisons() {
    use JValue::*;
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
}
