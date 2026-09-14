use crate::engines::{exec_error::{ExecError, ExecErrorKind, ExecResult}, interpreter::{interpreter_frame::InterpreterFrame, slot::Slot, step_outcome::{StepControl, StepOutcome}}};

// 0x60
pub fn iadd(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let Slot::Int(value2) = f.pop()? else {
        return Err(f.make_exec_error(ExecErrorKind::MismatchSlotType));
    };

    let Slot::Int(value1) = f.pop()? else {
        return Err(f.make_exec_error(ExecErrorKind::MismatchSlotType));
    };

    let res = value1.wrapping_add(value2);

    f.push(Slot::Int(res));

    Ok(StepControl::Continue)
}
