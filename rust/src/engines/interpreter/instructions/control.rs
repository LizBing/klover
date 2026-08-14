use crate::engines::{exec_error::{ExecError, ExecResult}, interpreter::{interpreter_frame::InterpreterFrame, step_outcome::StepOutcome}, slot::Slot};

// 0xac
pub fn ireturn(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let s = f.pop()?;

    match s {
        Slot::Int(_) => Ok(StepOutcome::Return(s)),
        _ => Err(ExecError::MismatchSlotType),
    }
}
