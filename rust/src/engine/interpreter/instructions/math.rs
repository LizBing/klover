use crate::engine::{
    engine_error::{ExecResult, JavaExceptionKind},
    interpreter::interpreter_frame::InterpreterFrame,
    outcome::{PendingException, StepOutcome},
    slot::Slot,
};

fn pop_ints(f: &mut InterpreterFrame) -> ExecResult<(i32, i32)> {
    let rhs = f.pop()?.as_int()?;
    let lhs = f.pop()?.as_int()?;
    Ok((lhs, rhs))
}

fn pop_longs(f: &mut InterpreterFrame) -> ExecResult<(i64, i64)> {
    let rhs = f.pop_long()?;
    let lhs = f.pop_long()?;
    Ok((lhs, rhs))
}

fn pop_floats(f: &mut InterpreterFrame) -> ExecResult<(f32, f32)> {
    let rhs = f.pop()?.as_float()?;
    let lhs = f.pop()?.as_float()?;
    Ok((lhs, rhs))
}

fn pop_doubles(f: &mut InterpreterFrame) -> ExecResult<(f64, f64)> {
    let rhs = f.pop_double()?;
    let lhs = f.pop_double()?;
    Ok((lhs, rhs))
}

fn arithmetic_exception() -> StepOutcome {
    StepOutcome::Throw(PendingException::JVMGen(
        JavaExceptionKind::ArithmeticException,
    ))
}

macro_rules! int_binary {
    ($name:ident, $op:expr) => {
        pub fn $name(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
            let (lhs, rhs) = pop_ints(f)?;
            f.push(Slot::int($op(lhs, rhs)))?;
            Ok(StepOutcome::Continue)
        }
    };
}

macro_rules! long_binary {
    ($name:ident, $op:expr) => {
        pub fn $name(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
            let (lhs, rhs) = pop_longs(f)?;
            f.push_long($op(lhs, rhs))?;
            Ok(StepOutcome::Continue)
        }
    };
}

macro_rules! float_binary {
    ($name:ident, $op:tt) => {
        pub fn $name(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
            let (lhs, rhs) = pop_floats(f)?;
            f.push(Slot::float(lhs $op rhs))?;
            Ok(StepOutcome::Continue)
        }
    };
}

macro_rules! double_binary {
    ($name:ident, $op:tt) => {
        pub fn $name(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
            let (lhs, rhs) = pop_doubles(f)?;
            f.push_double(lhs $op rhs)?;
            Ok(StepOutcome::Continue)
        }
    };
}

int_binary!(iadd, i32::wrapping_add);
long_binary!(ladd, i64::wrapping_add);
float_binary!(fadd, +);
double_binary!(dadd, +);

int_binary!(isub, i32::wrapping_sub);
long_binary!(lsub, i64::wrapping_sub);
float_binary!(fsub, -);
double_binary!(dsub, -);

int_binary!(imul, i32::wrapping_mul);
long_binary!(lmul, i64::wrapping_mul);
float_binary!(fmul, *);
double_binary!(dmul, *);

pub fn idiv(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let (lhs, rhs) = pop_ints(f)?;
    if rhs == 0 {
        return Ok(arithmetic_exception());
    }

    let result = if lhs == i32::MIN && rhs == -1 {
        i32::MIN
    } else {
        lhs / rhs
    };
    f.push(Slot::int(result))?;
    Ok(StepOutcome::Continue)
}

pub fn ldiv(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let (lhs, rhs) = pop_longs(f)?;
    if rhs == 0 {
        return Ok(arithmetic_exception());
    }

    let result = if lhs == i64::MIN && rhs == -1 {
        i64::MIN
    } else {
        lhs / rhs
    };
    f.push_long(result)?;
    Ok(StepOutcome::Continue)
}

float_binary!(fdiv, /);
double_binary!(ddiv, /);

pub fn irem(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let (lhs, rhs) = pop_ints(f)?;
    if rhs == 0 {
        return Ok(arithmetic_exception());
    }

    let result = if lhs == i32::MIN && rhs == -1 {
        0
    } else {
        lhs % rhs
    };
    f.push(Slot::int(result))?;
    Ok(StepOutcome::Continue)
}

pub fn lrem(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let (lhs, rhs) = pop_longs(f)?;
    if rhs == 0 {
        return Ok(arithmetic_exception());
    }

    let result = if lhs == i64::MIN && rhs == -1 {
        0
    } else {
        lhs % rhs
    };
    f.push_long(result)?;
    Ok(StepOutcome::Continue)
}

float_binary!(frem, %);
double_binary!(drem, %);

pub fn ineg(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let value = f.pop()?.as_int()?;
    f.push(Slot::int(value.wrapping_neg()))?;
    Ok(StepOutcome::Continue)
}

pub fn lneg(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let value = f.pop_long()?;
    f.push_long(value.wrapping_neg())?;
    Ok(StepOutcome::Continue)
}

pub fn fneg(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let value = f.pop()?.as_float()?;
    f.push(Slot::float(-value))?;
    Ok(StepOutcome::Continue)
}

pub fn dneg(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let value = f.pop_double()?;
    f.push_double(-value)?;
    Ok(StepOutcome::Continue)
}

pub fn ishl(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let (lhs, rhs) = pop_ints(f)?;
    f.push(Slot::int(lhs.wrapping_shl((rhs & 0x1f) as u32)))?;
    Ok(StepOutcome::Continue)
}

pub fn lshl(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let rhs = f.pop()?.as_int()?;
    let lhs = f.pop_long()?;
    f.push_long(lhs.wrapping_shl((rhs & 0x3f) as u32))?;
    Ok(StepOutcome::Continue)
}

pub fn ishr(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let (lhs, rhs) = pop_ints(f)?;
    f.push(Slot::int(lhs >> (rhs & 0x1f)))?;
    Ok(StepOutcome::Continue)
}

pub fn lshr(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let rhs = f.pop()?.as_int()?;
    let lhs = f.pop_long()?;
    f.push_long(lhs >> (rhs & 0x3f))?;
    Ok(StepOutcome::Continue)
}

pub fn iushr(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let (lhs, rhs) = pop_ints(f)?;
    let result = (lhs as u32) >> ((rhs & 0x1f) as u32);
    f.push(Slot::int(result as i32))?;
    Ok(StepOutcome::Continue)
}

pub fn lushr(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let rhs = f.pop()?.as_int()?;
    let lhs = f.pop_long()?;
    let result = (lhs as u64) >> ((rhs & 0x3f) as u32);
    f.push_long(result as i64)?;
    Ok(StepOutcome::Continue)
}

int_binary!(iand, |lhs: i32, rhs: i32| lhs & rhs);
long_binary!(land, |lhs: i64, rhs: i64| lhs & rhs);
int_binary!(ior, |lhs: i32, rhs: i32| lhs | rhs);
long_binary!(lor, |lhs: i64, rhs: i64| lhs | rhs);
int_binary!(ixor, |lhs: i32, rhs: i32| lhs ^ rhs);
long_binary!(lxor, |lhs: i64, rhs: i64| lhs ^ rhs);

pub fn iinc(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u8()? as usize;
    let increment = f.read_i8()? as i32;
    let value = f.get_local(index)?.as_int()?;
    f.set_local(index, Slot::int(value.wrapping_add(increment)))?;
    Ok(StepOutcome::Continue)
}
