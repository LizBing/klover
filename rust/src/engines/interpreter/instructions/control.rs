use crate::{engines::{exec_dispatcher::MethodReturn, exec_error::{ExecErrorKind, ExecResult}, interpreter::{interpreter_frame::InterpreterFrame, slot::Slot, step_outcome::{StepControl, StepOutcome}}}, oops::jvalue::JValue};

// 0xac
pub fn ireturn(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let s = f.pop()?;

    match s {
        Slot::Int(v) => Ok(StepControl::Return(MethodReturn::Value(JValue::Int(v)))),
        _ => Err(f.make_exec_error(ExecErrorKind::MismatchSlotType)),
    }
}
