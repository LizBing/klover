use crate::engines::{
    exec_error::ExecResult,
    interpreter::{interpreter_frame::InterpreterFrame, step_outcome::StepControl},
};
use crate::oops::jvalue::{JDouble, JFloat, JInt, JLong};

macro_rules! convert {
    ($name:ident, $pop:ident, $push:ident, $wrapper:ident, $target:ty) => {
        pub fn $name<'exec, 'meta: 'exec>(
            f: &mut InterpreterFrame<'exec, 'meta>,
        ) -> ExecResult<StepControl<'exec, 'meta>> {
            let value = f.$pop()?.0;
            f.$push($wrapper(value as $target));
            Ok(StepControl::Continue)
        }
    };
}

// Rust float-to-integer casts truncate, saturate on overflow, and map NaN to zero.
// Integer narrowing preserves the low bits, as required by JVM conversions.
convert!(i2l, pop_int, push_long, JLong, i64);
convert!(i2f, pop_int, push_float, JFloat, f32);
convert!(i2d, pop_int, push_double, JDouble, f64);
convert!(l2i, pop_long, push_int, JInt, i32);
convert!(l2f, pop_long, push_float, JFloat, f32);
convert!(l2d, pop_long, push_double, JDouble, f64);
convert!(f2i, pop_float, push_int, JInt, i32);
convert!(f2l, pop_float, push_long, JLong, i64);
convert!(f2d, pop_float, push_double, JDouble, f64);
convert!(d2i, pop_double, push_int, JInt, i32);
convert!(d2l, pop_double, push_long, JLong, i64);
convert!(d2f, pop_double, push_float, JFloat, f32);

pub fn i2b<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let value = f.pop_int()?.0;
    f.push_int(JInt(value as i8 as i32));
    Ok(StepControl::Continue)
}

pub fn i2c<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let value = f.pop_int()?.0;
    f.push_int(JInt(value as u16 as i32));
    Ok(StepControl::Continue)
}

pub fn i2s<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let value = f.pop_int()?.0;
    f.push_int(JInt(value as i16 as i32));
    Ok(StepControl::Continue)
}
