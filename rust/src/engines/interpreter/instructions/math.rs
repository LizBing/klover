use crate::engines::{exec_error::{ExecError, ExecResult}, interpreter::{interpreter_frame::InterpreterFrame, step_outcome::StepOutcome}, slot::Slot};

// 0x60
pub fn iadd(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let Slot::Int(value2) = f.pop()? else {
        return Err(ExecError::MismatchSlotType);
    };

    let Slot::Int(value1) = f.pop()? else {
        return Err(ExecError::MismatchSlotType);
    };

    let res = value1 + value2;

    f.push(Slot::Int(res));

    Ok(StepOutcome::Continue)
}
