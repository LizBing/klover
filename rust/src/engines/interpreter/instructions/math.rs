use crate::engines::{
    exec_error::{ExecErrorKind, ExecResult},
    interpreter::{interpreter_frame::InterpreterFrame, slot::Slot, step_outcome::StepControl},
};

macro_rules! binary {
    ($name:ident, $pop:ident, $push:ident, $op:expr) => {
        pub fn $name(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
            let right = f.$pop()?;
            let left = f.$pop()?;
            f.$push(($op)(left, right));
            Ok(StepControl::Continue)
        }
    };
}

macro_rules! unary {
    ($name:ident, $pop:ident, $push:ident, $op:expr) => {
        pub fn $name(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
            let value = f.$pop()?;
            f.$push(($op)(value));
            Ok(StepControl::Continue)
        }
    };
}
binary!(iadd, pop_int, push_int, i32::wrapping_add);
binary!(isub, pop_int, push_int, i32::wrapping_sub);
binary!(imul, pop_int, push_int, i32::wrapping_mul);

pub fn idiv(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let right = f.pop_int()?;
    let left = f.pop_int()?;
    if right == 0 {
        return Err(f.make_exec_error(ExecErrorKind::DivisionByZero));
    }
    f.push_int(left.wrapping_div(right));
    Ok(StepControl::Continue)
}

pub fn irem(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let right = f.pop_int()?;
    let left = f.pop_int()?;
    if right == 0 {
        return Err(f.make_exec_error(ExecErrorKind::DivisionByZero));
    }
    f.push_int(left.wrapping_rem(right));
    Ok(StepControl::Continue)
}
unary!(ineg, pop_int, push_int, i32::wrapping_neg);
binary!(iand, pop_int, push_int, |a: i32, b: i32| a & b);
binary!(ior, pop_int, push_int, |a: i32, b: i32| a | b);
binary!(ixor, pop_int, push_int, |a: i32, b: i32| a ^ b);

pub fn ishl(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let distance = f.pop_int()? as u32;
    let value = f.pop_int()?;
    f.push_int(value.wrapping_shl(distance));
    Ok(StepControl::Continue)
}

pub fn ishr(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let distance = f.pop_int()? as u32;
    let value = f.pop_int()?;
    f.push_int(value.wrapping_shr(distance));
    Ok(StepControl::Continue)
}

pub fn iushr(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let distance = f.pop_int()? as u32;
    let value = f.pop_int()?;
    f.push_int((value as u32).wrapping_shr(distance) as i32);
    Ok(StepControl::Continue)
}
binary!(ladd, pop_long, push_long, i64::wrapping_add);
binary!(lsub, pop_long, push_long, i64::wrapping_sub);
binary!(lmul, pop_long, push_long, i64::wrapping_mul);

pub fn ldiv(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let right = f.pop_long()?;
    let left = f.pop_long()?;
    if right == 0 {
        return Err(f.make_exec_error(ExecErrorKind::DivisionByZero));
    }
    f.push_long(left.wrapping_div(right));
    Ok(StepControl::Continue)
}

pub fn lrem(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let right = f.pop_long()?;
    let left = f.pop_long()?;
    if right == 0 {
        return Err(f.make_exec_error(ExecErrorKind::DivisionByZero));
    }
    f.push_long(left.wrapping_rem(right));
    Ok(StepControl::Continue)
}
unary!(lneg, pop_long, push_long, i64::wrapping_neg);
binary!(land, pop_long, push_long, |a: i64, b: i64| a & b);
binary!(lor, pop_long, push_long, |a: i64, b: i64| a | b);
binary!(lxor, pop_long, push_long, |a: i64, b: i64| a ^ b);

pub fn lshl(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let distance = f.pop_int()? as u32;
    let value = f.pop_long()?;
    f.push_long(value.wrapping_shl(distance));
    Ok(StepControl::Continue)
}

pub fn lshr(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let distance = f.pop_int()? as u32;
    let value = f.pop_long()?;
    f.push_long(value.wrapping_shr(distance));
    Ok(StepControl::Continue)
}

pub fn lushr(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let distance = f.pop_int()? as u32;
    let value = f.pop_long()?;
    f.push_long((value as u64).wrapping_shr(distance) as i64);
    Ok(StepControl::Continue)
}
binary!(fadd, pop_float, push_float, |a: f32, b: f32| a + b);
binary!(fsub, pop_float, push_float, |a: f32, b: f32| a - b);
binary!(fmul, pop_float, push_float, |a: f32, b: f32| a * b);
binary!(fdiv, pop_float, push_float, |a: f32, b: f32| a / b);
binary!(frem, pop_float, push_float, |a: f32, b: f32| a % b);
unary!(fneg, pop_float, push_float, |v: f32| -v);
binary!(dadd, pop_double, push_double, |a: f64, b: f64| a + b);
binary!(dsub, pop_double, push_double, |a: f64, b: f64| a - b);
binary!(dmul, pop_double, push_double, |a: f64, b: f64| a * b);
binary!(ddiv, pop_double, push_double, |a: f64, b: f64| a / b);
binary!(drem, pop_double, push_double, |a: f64, b: f64| a % b);
unary!(dneg, pop_double, push_double, |v: f64| -v);

pub fn iinc(f: &mut InterpreterFrame, idx: usize, amount: i16) -> ExecResult<StepControl> {
    let Slot::Int(value) = f.get_local(idx)? else {
        return Err(f.make_exec_error(ExecErrorKind::MismatchSlotType));
    };
    f.set_local(idx, Slot::Int(value.wrapping_add(i32::from(amount))))?;
    Ok(StepControl::Continue)
}
