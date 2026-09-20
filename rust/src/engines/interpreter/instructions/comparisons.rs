use super::control::branch;
use crate::code::instructions::InstIdx;
use crate::engines::{
    exec_error::ExecResult,
    interpreter::{interpreter_frame::InterpreterFrame, step_outcome::StepControl},
};

pub fn lcmp(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let right = f.pop_long()?;
    let left = f.pop_long()?;
    f.push_int(if left < right {
        -1
    } else if left > right {
        1
    } else {
        0
    });
    Ok(StepControl::Continue)
}

macro_rules! float_compare {
    ($name:ident, $pop:ident) => {
        pub fn $name(f: &mut InterpreterFrame, nan: i32) -> ExecResult<StepControl> {
            let right = f.$pop()?;
            let left = f.$pop()?;
            let result = if left.is_nan() || right.is_nan() {
                nan
            } else if left < right {
                -1
            } else if left > right {
                1
            } else {
                0
            };
            f.push_int(result);
            Ok(StepControl::Continue)
        }
    };
}
float_compare!(fcmp, pop_float);
float_compare!(dcmp, pop_double);

macro_rules! int_branch {
    ($zero:ident, $pair:ident, $op:tt) => {
        pub fn $zero(f: &mut InterpreterFrame, target: InstIdx) -> ExecResult<StepControl> {
            let value = f.pop_int()?;
            branch(f, target, value $op 0)
        }
        pub fn $pair(f: &mut InterpreterFrame, target: InstIdx) -> ExecResult<StepControl> {
            let right = f.pop_int()?;
            let left = f.pop_int()?;
            branch(f, target, left $op right)
        }
    };
}
int_branch!(ifeq, if_icmpeq, ==);
int_branch!(ifne, if_icmpne, !=);
int_branch!(iflt, if_icmplt, <);
int_branch!(ifge, if_icmpge, >=);
int_branch!(ifgt, if_icmpgt, >);
int_branch!(ifle, if_icmple, <=);

pub fn if_acmp(f: &mut InterpreterFrame, target: InstIdx, equal: bool) -> ExecResult<StepControl> {
    let right = f.pop_ref()?;
    let left = f.pop_ref()?;
    branch(f, target, (left.as_u32() == right.as_u32()) == equal)
}

pub fn if_null(f: &mut InterpreterFrame, target: InstIdx, null: bool) -> ExecResult<StepControl> {
    let value = f.pop_ref()?;
    branch(f, target, value.is_null() == null)
}
