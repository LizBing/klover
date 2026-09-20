use crate::engines::{
    exec_error::ExecResult,
    interpreter::{interpreter_frame::InterpreterFrame, step_outcome::StepControl},
};

macro_rules! convert {
    ($name:ident, $pop:ident, $push:ident, $target:ty) => {
        pub fn $name(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
            let value = f.$pop()?;
            f.$push(value as $target);
            Ok(StepControl::Continue)
        }
    };
}

// Rust float-to-integer casts truncate, saturate on overflow, and map NaN to zero.
// Integer narrowing preserves the low bits, as required by JVM conversions.
convert!(i2l, pop_int, push_long, i64);
convert!(i2f, pop_int, push_float, f32);
convert!(i2d, pop_int, push_double, f64);
convert!(l2i, pop_long, push_int, i32);
convert!(l2f, pop_long, push_float, f32);
convert!(l2d, pop_long, push_double, f64);
convert!(f2i, pop_float, push_int, i32);
convert!(f2l, pop_float, push_long, i64);
convert!(f2d, pop_float, push_double, f64);
convert!(d2i, pop_double, push_int, i32);
convert!(d2l, pop_double, push_long, i64);
convert!(d2f, pop_double, push_float, f32);

pub fn i2b(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let value = f.pop_int()?;
    f.push_int(value as i8 as i32);
    Ok(StepControl::Continue)
}

pub fn i2c(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let value = f.pop_int()?;
    f.push_int(value as u16 as i32);
    Ok(StepControl::Continue)
}

pub fn i2s(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let value = f.pop_int()?;
    f.push_int(value as i16 as i32);
    Ok(StepControl::Continue)
}
