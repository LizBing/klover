use crate::engines::{
    exec_error::{ExecErrorKind, ExecResult},
    interpreter::{interpreter_frame::InterpreterFrame, slot::Slot, step_outcome::StepControl},
};
use crate::oops::jvalue::{JDouble, JFloat, JInt, JLong, JShort};

macro_rules! binary {
    ($name:ident, $pop:ident, $push:ident, $wrapper:ident, $op:expr) => {
        pub fn $name<'exec, 'meta: 'exec>(
            f: &mut InterpreterFrame<'exec, 'meta>,
        ) -> ExecResult<StepControl<'exec, 'meta>> {
            let right = f.$pop()?.0;
            let left = f.$pop()?.0;
            f.$push($wrapper(($op)(left, right)));
            Ok(StepControl::Continue)
        }
    };
}

macro_rules! unary {
    ($name:ident, $pop:ident, $push:ident, $wrapper:ident, $op:expr) => {
        pub fn $name<'exec, 'meta: 'exec>(
            f: &mut InterpreterFrame<'exec, 'meta>,
        ) -> ExecResult<StepControl<'exec, 'meta>> {
            let value = f.$pop()?.0;
            f.$push($wrapper(($op)(value)));
            Ok(StepControl::Continue)
        }
    };
}
binary!(iadd, pop_int, push_int, JInt, i32::wrapping_add);
binary!(isub, pop_int, push_int, JInt, i32::wrapping_sub);
binary!(imul, pop_int, push_int, JInt, i32::wrapping_mul);

pub fn idiv<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let right = f.pop_int()?.0;
    let left = f.pop_int()?.0;
    if right == 0 {
        return Err(f.make_exec_error(ExecErrorKind::DivisionByZero));
    }
    f.push_int(JInt(left.wrapping_div(right)));
    Ok(StepControl::Continue)
}

pub fn irem<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let right = f.pop_int()?.0;
    let left = f.pop_int()?.0;
    if right == 0 {
        return Err(f.make_exec_error(ExecErrorKind::DivisionByZero));
    }
    f.push_int(JInt(left.wrapping_rem(right)));
    Ok(StepControl::Continue)
}
unary!(ineg, pop_int, push_int, JInt, i32::wrapping_neg);
binary!(iand, pop_int, push_int, JInt, |a: i32, b: i32| a & b);
binary!(ior, pop_int, push_int, JInt, |a: i32, b: i32| a | b);
binary!(ixor, pop_int, push_int, JInt, |a: i32, b: i32| a ^ b);

pub fn ishl<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let distance = f.pop_int()?.0 as u32;
    let value = f.pop_int()?.0;
    f.push_int(JInt(value.wrapping_shl(distance)));
    Ok(StepControl::Continue)
}

pub fn ishr<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let distance = f.pop_int()?.0 as u32;
    let value = f.pop_int()?.0;
    f.push_int(JInt(value.wrapping_shr(distance)));
    Ok(StepControl::Continue)
}

pub fn iushr<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let distance = f.pop_int()?.0 as u32;
    let value = f.pop_int()?.0;
    f.push_int(JInt((value as u32).wrapping_shr(distance) as i32));
    Ok(StepControl::Continue)
}
binary!(ladd, pop_long, push_long, JLong, i64::wrapping_add);
binary!(lsub, pop_long, push_long, JLong, i64::wrapping_sub);
binary!(lmul, pop_long, push_long, JLong, i64::wrapping_mul);

pub fn ldiv<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let right = f.pop_long()?.0;
    let left = f.pop_long()?.0;
    if right == 0 {
        return Err(f.make_exec_error(ExecErrorKind::DivisionByZero));
    }
    f.push_long(JLong(left.wrapping_div(right)));
    Ok(StepControl::Continue)
}

pub fn lrem<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let right = f.pop_long()?.0;
    let left = f.pop_long()?.0;
    if right == 0 {
        return Err(f.make_exec_error(ExecErrorKind::DivisionByZero));
    }
    f.push_long(JLong(left.wrapping_rem(right)));
    Ok(StepControl::Continue)
}
unary!(lneg, pop_long, push_long, JLong, i64::wrapping_neg);
binary!(land, pop_long, push_long, JLong, |a: i64, b: i64| a & b);
binary!(lor, pop_long, push_long, JLong, |a: i64, b: i64| a | b);
binary!(lxor, pop_long, push_long, JLong, |a: i64, b: i64| a ^ b);

pub fn lshl<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let distance = f.pop_int()?.0 as u32;
    let value = f.pop_long()?.0;
    f.push_long(JLong(value.wrapping_shl(distance)));
    Ok(StepControl::Continue)
}

pub fn lshr<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let distance = f.pop_int()?.0 as u32;
    let value = f.pop_long()?.0;
    f.push_long(JLong(value.wrapping_shr(distance)));
    Ok(StepControl::Continue)
}

pub fn lushr<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let distance = f.pop_int()?.0 as u32;
    let value = f.pop_long()?.0;
    f.push_long(JLong((value as u64).wrapping_shr(distance) as i64));
    Ok(StepControl::Continue)
}
binary!(fadd, pop_float, push_float, JFloat, |a: f32, b: f32| a + b);
binary!(fsub, pop_float, push_float, JFloat, |a: f32, b: f32| a - b);
binary!(fmul, pop_float, push_float, JFloat, |a: f32, b: f32| a * b);
binary!(fdiv, pop_float, push_float, JFloat, |a: f32, b: f32| a / b);
binary!(frem, pop_float, push_float, JFloat, |a: f32, b: f32| a % b);
unary!(fneg, pop_float, push_float, JFloat, |v: f32| -v);
binary!(dadd, pop_double, push_double, JDouble, |a: f64, b: f64| a
    + b);
binary!(dsub, pop_double, push_double, JDouble, |a: f64, b: f64| a
    - b);
binary!(dmul, pop_double, push_double, JDouble, |a: f64, b: f64| a
    * b);
binary!(ddiv, pop_double, push_double, JDouble, |a: f64, b: f64| a
    / b);
binary!(drem, pop_double, push_double, JDouble, |a: f64, b: f64| a
    % b);
unary!(dneg, pop_double, push_double, JDouble, |v: f64| -v);

pub fn iinc<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    idx: usize,
    JShort(amount): JShort,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let Slot::Int(JInt(value)) = f.get_local(idx)? else {
        return Err(f.make_exec_error(ExecErrorKind::MismatchSlotType));
    };
    f.set_local(idx, Slot::Int(JInt(value.wrapping_add(i32::from(amount)))))?;
    Ok(StepControl::Continue)
}
