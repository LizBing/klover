use crate::tests::support::{assert_value, class_bytes, ensure_metaspace, parse_class, run_method};
use crate::{
    class_loader::class_loader_data::ClassLoaderData,
    code::instructions::{
        InstIdx,
        Instruction::{self, *},
        LocalIdx,
    },
    engines::{
        exec_dispatcher::ExecDispatcher,
        exec_error::ExecErrorKind,
        interpreter::{
            interpreter::Interpreter, interpreter_frame::InterpreterFrame, slot::Slot,
            step_outcome::StepControl,
        },
        invocation::Invocation,
        java_frame::JavaFrame,
        protocol::{EngineExit, ExecBudget, MethodReturn},
    },
    gc_bindings::oop_hierarchy::NObjPtr,
    oops::jvalue::JValue,
    runtime::{
        java_thread::{JavaThread, JavaThreadState},
        ms_api::MsAllocator,
    },
};

fn with_code(code: &[u8], test: impl FnOnce(&mut InterpreterFrame<'_>)) {
    ensure_metaspace();
    let arena = MsAllocator::new();
    let cld = ClassLoaderData::new(&arena);
    let owner = parse_class(&cld, &class_bytes("()V", code, 16, 16));
    let call = Invocation::try_new(&owner, "test", "()V", &[]).unwrap();
    test(&mut InterpreterFrame::new(call));
}

fn with_frame(test: impl FnOnce(&mut InterpreterFrame<'_>)) {
    with_code(&[0xb1], test);
}

fn step<'a>(frame: &mut InterpreterFrame<'a>, inst: Instruction<'a>) {
    // These instructions own no borrowed switch tables.
    Interpreter::execute_instruction(frame, inst).unwrap();
}

fn assert_stack(frame: &InterpreterFrame<'_>, expected: &[Slot]) {
    let actual = frame.operand_stack();
    assert_eq!(
        actual.len(),
        expected.len(),
        "actual={actual:?}, expected={expected:?}"
    );
    for (actual, expected) in actual.iter().zip(expected) {
        let equal = match (actual, expected) {
            (Slot::Int(a), Slot::Int(b)) => a == b,
            (Slot::Float(a), Slot::Float(b)) => a.to_bits() == b.to_bits(),
            (Slot::Ref(a), Slot::Ref(b)) => a.as_u32() == b.as_u32(),
            (Slot::LongHigh(a), Slot::LongHigh(b))
            | (Slot::LongLow(a), Slot::LongLow(b))
            | (Slot::DoubleHigh(a), Slot::DoubleHigh(b))
            | (Slot::DoubleLow(a), Slot::DoubleLow(b)) => a == b,
            (Slot::Unused, Slot::Unused) => true,
            _ => false,
        };
        assert!(equal, "expected slot {expected:?}, got {actual:?}");
    }
}

#[test]
fn integer_binary_operations_cover_boundaries_and_operand_order() {
    let cases = [
        (IAdd, i32::MAX, 1, i32::MIN),
        (ISub, 7, 12, -5),
        (IMul, i32::MAX, 2, -2),
        (IDiv, -7, 3, -2),
        (IDiv, i32::MIN, -1, i32::MIN),
        (IRem, -7, 3, -1),
        (IRem, i32::MIN, -1, 0),
        (IAnd, 0b1100, 0b1010, 8),
        (IOr, 0b1100, 0b1010, 14),
        (IXor, 0b1100, 0b1010, 6),
        (IShL, 1, 32, 1),
        (IShL, 1, -1, i32::MIN),
        (IShR, -8, 1, -4),
        (IUShR, -1, 1, i32::MAX),
        (IUShR, -1, 33, i32::MAX),
    ];
    for (inst, left, right, expected) in cases {
        with_frame(|frame| {
            frame.push_int(left);
            frame.push_int(right);
            step(frame, inst);
            assert_eq!(frame.pop_int().unwrap(), expected);
            assert_stack(frame, &[]);
        });
    }
}

#[test]
fn long_binary_operations_cover_boundaries_and_operand_order() {
    for (inst, left, right, expected) in [
        (LAdd, i64::MAX, 1, i64::MIN),
        (LSub, 7, 12, -5),
        (LMul, i64::MAX, 2, -2),
        (LDiv, -7, 3, -2),
        (LDiv, i64::MIN, -1, i64::MIN),
        (LRem, -7, 3, -1),
        (LRem, i64::MIN, -1, 0),
        (LAnd, 12, 10, 8),
        (LOr, 12, 10, 14),
        (LXor, 12, 10, 6),
    ] {
        with_frame(|frame| {
            frame.push_long(left);
            frame.push_long(right);
            step(frame, inst);
            assert_eq!(frame.pop_long().unwrap(), expected);
            assert_stack(frame, &[]);
        });
    }
    for (inst, value, distance, expected) in [
        (LShL, 1, 64, 1),
        (LShL, 1, -1, i64::MIN),
        (LShR, -8, 1, -4),
        (LUShR, -1, 1, i64::MAX),
        (LUShR, -1, 65, i64::MAX),
    ] {
        with_frame(|frame| {
            frame.push_long(value);
            frame.push_int(distance);
            step(frame, inst);
            assert_eq!(frame.pop_long().unwrap(), expected);
        });
    }
}

#[test]
fn division_and_remainder_by_zero_are_structured_errors() {
    for inst in [IDiv, IRem, LDiv, LRem] {
        with_frame(|frame| {
            if matches!(inst, LDiv | LRem) {
                frame.push_long(1);
                frame.push_long(0);
            } else {
                frame.push_int(1);
                frame.push_int(0);
            }
            let error = Interpreter::execute_instruction(frame, inst).err().unwrap();
            assert!(matches!(error.kind, ExecErrorKind::DivisionByZero));
            assert_eq!(error.brief.unwrap().method, "test");
        });
    }
}

#[test]
fn floating_arithmetic_and_signed_zero() {
    for (inst, expected) in [
        (FAdd, 9.0),
        (FSub, 5.0),
        (FMul, 14.0),
        (FDiv, 3.5),
        (FRem, 1.0),
    ] {
        with_frame(|f| {
            f.push_float(7.0);
            f.push_float(2.0);
            step(f, inst);
            assert_eq!(f.pop_float().unwrap(), expected);
        });
    }
    for (inst, expected) in [
        (DAdd, 9.0),
        (DSub, 5.0),
        (DMul, 14.0),
        (DDiv, 3.5),
        (DRem, 1.0),
    ] {
        with_frame(|f| {
            f.push_double(7.0);
            f.push_double(2.0);
            step(f, inst);
            assert_eq!(f.pop_double().unwrap(), expected);
        });
    }
    with_frame(|f| {
        f.push_float(0.0);
        step(f, FNeg);
        assert_eq!(f.pop_float().unwrap().to_bits(), (-0.0f32).to_bits());
        f.push_double(0.0);
        step(f, DNeg);
        assert_eq!(f.pop_double().unwrap().to_bits(), (-0.0f64).to_bits());
        f.push_int(i32::MIN);
        step(f, INeg);
        assert_eq!(f.pop_int().unwrap(), i32::MIN);
        f.push_long(i64::MIN);
        step(f, LNeg);
        assert_eq!(f.pop_long().unwrap(), i64::MIN);
    });
}

fn push_value(f: &mut InterpreterFrame<'_>, value: JValue) {
    match value {
        JValue::Int(v) => f.push_int(v),
        JValue::Long(v) => f.push_long(v),
        JValue::Float(v) => f.push_float(v),
        JValue::Double(v) => f.push_double(v),
        JValue::Ref(v) => f.push_ref(v),
        _ => panic!("unsupported test value"),
    }
}

#[test]
fn numeric_conversions_preserve_java_rules() {
    use JValue::{Double, Float, Int, Long};
    let cases = [
        (I2L, Int(-3), Long(-3)),
        (I2F, Int(-3), Float(-3.0)),
        (I2D, Int(-3), Double(-3.0)),
        (L2I, Long(0x1_0000_0001), Int(1)),
        (L2F, Long(-3), Float(-3.0)),
        (L2D, Long(-3), Double(-3.0)),
        (F2I, Float(-3.75), Int(-3)),
        (F2L, Float(-3.75), Long(-3)),
        (F2D, Float(1.5), Double(1.5)),
        (D2I, Double(-3.75), Int(-3)),
        (D2L, Double(-3.75), Long(-3)),
        (D2F, Double(1.5), Float(1.5)),
        (I2B, Int(255), Int(-1)),
        (I2C, Int(-1), Int(65535)),
        (I2S, Int(65535), Int(-1)),
        (F2I, Float(f32::NAN), Int(0)),
        (F2I, Float(f32::INFINITY), Int(i32::MAX)),
        (D2I, Double(f64::NEG_INFINITY), Int(i32::MIN)),
        (D2L, Double(f64::INFINITY), Long(i64::MAX)),
        (D2L, Double(f64::NAN), Long(0)),
    ];
    for (inst, input, expected) in cases {
        with_frame(|f| {
            push_value(f, input);
            step(f, inst);
            let actual = match expected {
                Int(_) => Int(f.pop_int().unwrap()),
                Long(_) => Long(f.pop_long().unwrap()),
                Float(_) => Float(f.pop_float().unwrap()),
                Double(_) => Double(f.pop_double().unwrap()),
                _ => unreachable!(),
            };
            assert_value(MethodReturn::Value(actual), expected);
            assert_stack(f, &[]);
        });
    }
}

#[test]
fn comparisons_cover_nan_ordering_and_equal_zero() {
    for (inst, left, right, expected) in [
        (FCmpL, f32::NAN, 0.0, -1),
        (FCmpG, f32::NAN, 0.0, 1),
        (FCmpL, -0.0, 0.0, 0),
        (FCmpG, 3.0, 2.0, 1),
        (FCmpG, 1.0, 2.0, -1),
    ] {
        with_frame(|f| {
            f.push_float(left);
            f.push_float(right);
            step(f, inst);
            assert_eq!(f.pop_int().unwrap(), expected);
        });
    }
    for (inst, left, right, expected) in [
        (DCmpL, 0.0, f64::NAN, -1),
        (DCmpG, 0.0, f64::NAN, 1),
        (DCmpL, -0.0, 0.0, 0),
        (DCmpG, 3.0, 2.0, 1),
        (DCmpG, 1.0, 2.0, -1),
    ] {
        with_frame(|f| {
            f.push_double(left);
            f.push_double(right);
            step(f, inst);
            assert_eq!(f.pop_int().unwrap(), expected);
        });
    }
    for (left, right, expected) in [(i64::MIN, i64::MAX, -1), (4, 4, 0), (5, 4, 1)] {
        with_frame(|f| {
            f.push_long(left);
            f.push_long(right);
            step(f, LCmp);
            assert_eq!(f.pop_int().unwrap(), expected);
        });
    }
}

#[test]
fn constants_and_typed_returns() {
    use JValue::{Double, Float, Int, Long, Ref};
    for (inst, ret, expected) in [
        (IConst(-1), IReturn, Int(-1)),
        (BIPush(-128), IReturn, Int(-128)),
        (SIPush(-32768), IReturn, Int(-32768)),
        (LConst(1), LReturn, Long(1)),
        (FConst(2.0), FReturn, Float(2.0)),
        (DConst(1.0), DReturn, Double(1.0)),
        (AConstNull, AReturn, Ref(NObjPtr::null())),
    ] {
        with_frame(|f| {
            step(f, inst);
            let StepControl::Return(value) = Interpreter::execute_instruction(f, ret).unwrap()
            else {
                panic!("did not return")
            };
            assert_value(value, expected);
        });
    }
    with_frame(|f| {
        step(f, NOp);
        assert_stack(f, &[]);
        assert!(matches!(
            Interpreter::execute_instruction(f, Return).unwrap(),
            StepControl::Return(MethodReturn::Void)
        ));
    });
}

#[test]
fn stack_forms_preserve_value_order() {
    use Slot::{DoubleHigh, DoubleLow, Int, LongHigh, LongLow};
    let h = LongHigh([0, 0, 0, 1]);
    let l = LongLow([0, 0, 0, 2]);
    let dh = DoubleHigh([0, 0, 0, 3]);
    let dl = DoubleLow([0, 0, 0, 4]);
    let cases: Vec<(Instruction<'_>, Vec<Slot>, Vec<Slot>)> = vec![
        (Pop, vec![Int(1), Int(2)], vec![Int(1)]),
        (Pop2, vec![Int(1), Int(2)], vec![]),
        (Pop2, vec![h, l], vec![]),
        (Dup, vec![Int(1)], vec![Int(1), Int(1)]),
        (DupX1, vec![Int(1), Int(2)], vec![Int(2), Int(1), Int(2)]),
        (
            DupX2,
            vec![Int(1), Int(2), Int(3)],
            vec![Int(3), Int(1), Int(2), Int(3)],
        ),
        (DupX2, vec![h, l, Int(3)], vec![Int(3), h, l, Int(3)]),
        (
            Dup2,
            vec![Int(1), Int(2)],
            vec![Int(1), Int(2), Int(1), Int(2)],
        ),
        (Dup2, vec![h, l], vec![h, l, h, l]),
        (
            Dup2X1,
            vec![Int(1), Int(2), Int(3)],
            vec![Int(2), Int(3), Int(1), Int(2), Int(3)],
        ),
        (Dup2X1, vec![Int(1), h, l], vec![h, l, Int(1), h, l]),
        (
            Dup2X2,
            vec![Int(1), Int(2), Int(3), Int(4)],
            vec![Int(3), Int(4), Int(1), Int(2), Int(3), Int(4)],
        ),
        (
            Dup2X2,
            vec![Int(1), Int(2), h, l],
            vec![h, l, Int(1), Int(2), h, l],
        ),
        (
            Dup2X2,
            vec![h, l, Int(1), Int(2)],
            vec![Int(1), Int(2), h, l, Int(1), Int(2)],
        ),
        (Dup2X2, vec![h, l, dh, dl], vec![dh, dl, h, l, dh, dl]),
        (Swap, vec![Int(1), Int(2)], vec![Int(2), Int(1)]),
    ];
    for (inst, before, expected) in cases {
        with_frame(|f| {
            for slot in before {
                f.push(slot);
            }
            step(f, inst);
            assert_stack(f, &expected);
        });
    }
}

#[test]
fn invalid_stack_forms_fail_without_partial_mutation() {
    let h = Slot::LongHigh([0; 4]);
    let l = Slot::LongLow([0; 4]);
    for (inst, before) in [
        (Pop, vec![h, l]),
        (Dup, vec![h, l]),
        (Swap, vec![h, l]),
        (Pop2, vec![Slot::Int(1), l]),
        (Dup2, vec![l, h]),
        (DupX1, vec![Slot::Int(1)]),
        (Dup2X2, vec![h, l]),
    ] {
        with_frame(|f| {
            for &slot in &before {
                f.push(slot);
            }
            let error = Interpreter::execute_instruction(f, inst).err().unwrap();
            assert!(matches!(
                error.kind,
                ExecErrorKind::MismatchSlotType | ExecErrorKind::StackUnderflow
            ));
            assert_stack(f, &before);
        });
    }
}

#[test]
fn local_loads_stores_and_signed_increment() {
    with_frame(|f| {
        f.push_int(i32::MAX);
        step(f, IStore(LocalIdx(3)));
        step(f, IInc(LocalIdx(3), 1));
        step(f, ILoad(LocalIdx(3)));
        assert_eq!(f.pop_int().unwrap(), i32::MIN);
        step(f, IInc(LocalIdx(3), -1));
        step(f, ILoad(LocalIdx(3)));
        assert_eq!(f.pop_int().unwrap(), i32::MAX);
        f.push_long(i64::MIN);
        step(f, LStore(LocalIdx(4)));
        step(f, LLoad(LocalIdx(4)));
        assert_eq!(f.pop_long().unwrap(), i64::MIN);
        f.push_double(-0.0);
        step(f, DStore(LocalIdx(6)));
        step(f, DLoad(LocalIdx(6)));
        assert_eq!(f.pop_double().unwrap().to_bits(), (-0.0f64).to_bits());
        f.push_float(1.5);
        step(f, FStore(LocalIdx(8)));
        step(f, FLoad(LocalIdx(8)));
        assert_eq!(f.pop_float().unwrap(), 1.5);
        f.push_ref(NObjPtr::null());
        step(f, AStore(LocalIdx(9)));
        step(f, ALoad(LocalIdx(9)));
        assert!(f.pop_ref().unwrap().is_null());
        assert_stack(f, &[]);
    });
}

#[test]
fn overwriting_either_wide_local_half_invalidates_its_partner() {
    for idx in [4, 5] {
        with_frame(|f| {
            f.push_long(42);
            step(f, LStore(LocalIdx(4)));
            f.set_local(idx, Slot::Int(7)).unwrap();
            assert!(matches!(
                f.get_local(if idx == 4 { 5 } else { 4 }).unwrap(),
                Slot::Unused
            ));
            f.push_double(3.0);
            step(f, DStore(LocalIdx(4)));
            f.push_long(99);
            step(f, LStore(LocalIdx(5)));
            assert!(matches!(f.get_local(4).unwrap(), Slot::Unused));
            step(f, LLoad(LocalIdx(5)));
            assert_eq!(f.pop_long().unwrap(), 99);
        });
    }
}

#[test]
fn invalid_locals_and_types_report_structured_errors() {
    with_frame(|f| {
        assert!(matches!(
            f.get_local(16).unwrap_err().kind,
            ExecErrorKind::InvalidLocalIndex(16)
        ));
        let error = Interpreter::execute_instruction(f, ILoad(LocalIdx(0)))
            .err()
            .unwrap();
        assert!(matches!(error.kind, ExecErrorKind::MismatchSlotType));
        f.set_local(15, Slot::Int(42)).unwrap();
        let error = f
            .set_wide_local(15, Slot::LongHigh([0; 4]), Slot::LongLow([0; 4]))
            .unwrap_err();
        assert!(matches!(error.kind, ExecErrorKind::InvalidLocalIndex(16)));
        assert!(matches!(f.get_local(15).unwrap(), Slot::Int(42)));
        assert!(matches!(
            f.pop_int().unwrap_err().kind,
            ExecErrorKind::StackUnderflow
        ));
        f.push_float(1.0);
        assert!(matches!(
            f.pop_int().unwrap_err().kind,
            ExecErrorKind::MismatchSlotType
        ));
    });
}

#[test]
fn conditional_branches_cover_taken_and_untaken_paths() {
    let branches: [(fn(InstIdx) -> Instruction<'static>, i32, i32); 6] = [
        (IfEq, 0, 1),
        (IfNe, 1, 0),
        (IfLt, -1, 0),
        (IfGe, 0, -1),
        (IfGt, 1, 0),
        (IfLe, 0, 1),
    ];
    for (branch, taken, untaken) in branches {
        for (value, expected) in [(taken, 2), (untaken, 1)] {
            with_code(&[0x04, 0x05, 0xb1], |f| {
                f.push_int(value);
                step(f, branch(InstIdx(1)));
                assert!(matches!(f.next_inst(), Some(IConst(v)) if v == expected));
            });
        }
    }
    let pairs: [(fn(InstIdx) -> Instruction<'static>, i32, i32); 6] = [
        (IfICmpEq, 5, 6),
        (IfICmpNe, 6, 5),
        (IfICmpLt, 4, 5),
        (IfICmpGe, 5, 4),
        (IfICmpGt, 6, 5),
        (IfICmpLe, 5, 6),
    ];
    for (branch, taken, untaken) in pairs {
        for (left, expected) in [(taken, 2), (untaken, 1)] {
            with_code(&[0x04, 0x05, 0xb1], |f| {
                f.push_int(left);
                f.push_int(5);
                step(f, branch(InstIdx(1)));
                assert!(matches!(f.next_inst(), Some(IConst(v)) if v == expected));
            });
        }
    }
}

#[test]
fn reference_branches_and_invalid_targets() {
    for (inst, expected) in [(IfNull(InstIdx(1)), 2), (IfNonNull(InstIdx(1)), 1)] {
        with_code(&[0x04, 0x05, 0xb1], |f| {
            f.push_ref(NObjPtr::null());
            step(f, inst);
            assert!(matches!(f.next_inst(), Some(IConst(v)) if v == expected));
        });
    }
    for (inst, expected) in [(IfACmpEq(InstIdx(1)), 2), (IfACmpNe(InstIdx(1)), 1)] {
        with_code(&[0x04, 0x05, 0xb1], |f| {
            f.push_ref(NObjPtr::null());
            f.push_ref(NObjPtr::null());
            step(f, inst);
            assert!(matches!(f.next_inst(), Some(IConst(v)) if v == expected));
        });
    }
    with_frame(|f| {
        assert!(matches!(
            f.jump_to(InstIdx(1)).unwrap_err().kind,
            ExecErrorKind::InvalidBranchTarget(1)
        ));
        step(f, Goto(InstIdx(0)));
        assert!(matches!(f.next_inst(), Some(Return)));
    });
}

#[test]
fn budget_zero_and_instruction_cost_preserve_resumption() {
    with_code(&[0x04, 0x05, 0x60, 0xac], |f| {
        assert!(matches!(
            f.resume(&mut ExecBudget::new(0)).unwrap(),
            EngineExit::BudgetExhausted
        ));
        assert_stack(f, &[]);
        assert!(matches!(
            f.resume(&mut ExecBudget::new(2)).unwrap(),
            EngineExit::BudgetExhausted
        ));
        assert_stack(f, &[Slot::Int(1), Slot::Int(2)]);
        // iadd currently costs two but may start with one unit remaining.
        let mut budget = ExecBudget::new(1);
        assert!(matches!(
            f.resume(&mut budget).unwrap(),
            EngineExit::BudgetExhausted
        ));
        assert_eq!(budget.remaining(), 0);
        assert_stack(f, &[Slot::Int(3)]);
        let EngineExit::Return(value) = f.resume(&mut ExecBudget::new(1)).unwrap() else {
            panic!("return lost after yield")
        };
        assert_value(value, JValue::Int(3));
    });
}

#[test]
fn unsupported_instruction_and_unexpected_eof_are_errors() {
    with_frame(|f| {
        assert!(matches!(
            Interpreter::execute_instruction(f, Unsupported)
                .err()
                .unwrap()
                .kind,
            ExecErrorKind::UnsupportedInstruction
        ));
    });
    with_code(&[0x00], |f| {
        assert!(matches!(
            f.resume(&mut ExecBudget::new(2)).err().unwrap().kind,
            ExecErrorKind::UnexpectedEOF
        ));
    });
}

#[test]
fn dispatcher_without_frame_reports_no_frame() {
    let mut thread = JavaThread::new();
    assert!(matches!(thread.state(), JavaThreadState::New));
    let error = ExecDispatcher::poll(&mut thread, &mut ExecBudget::new(10))
        .err()
        .unwrap();
    assert!(matches!(error.kind, ExecErrorKind::NoFrame));
}

#[test]
fn wide_bytecode_survives_parsing_lowering_and_execution() {
    ensure_metaspace();
    let arena = MsAllocator::new();
    let cld = ClassLoaderData::new(&arena);
    // iload_0; wide istore 300; wide iinc 300,-1000; wide iload 300; ireturn.
    let owner = parse_class(
        &cld,
        &class_bytes(
            "(I)I",
            &[
                0x1a, 0xc4, 0x36, 0x01, 0x2c, 0xc4, 0x84, 0x01, 0x2c, 0xfc, 0x18, 0xc4, 0x15, 0x01,
                0x2c, 0xac,
            ],
            2,
            301,
        ),
    );
    assert_value(
        run_method(&owner, "test", "(I)I", &[JValue::Int(10)], 1),
        JValue::Int(-990),
    );
    // lload_0; wide lstore 400; wide lload 400; lreturn.
    let owner = parse_class(
        &cld,
        &class_bytes(
            "(J)J",
            &[0x1e, 0xc4, 0x37, 0x01, 0x90, 0xc4, 0x16, 0x01, 0x90, 0xad],
            2,
            402,
        ),
    );
    assert_value(
        run_method(&owner, "test", "(J)J", &[JValue::Long(1 << 40)], 1),
        JValue::Long(1 << 40),
    );
}
