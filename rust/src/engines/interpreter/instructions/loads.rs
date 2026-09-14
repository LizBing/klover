use crate::engines::{exec_error::ExecResult, interpreter::{interpreter_frame::InterpreterFrame, step_outcome::{StepControl, StepOutcome}}};

// 0x1a..=0x1d
pub fn iload(f: &mut InterpreterFrame, idx: usize) -> ExecResult<StepControl> {
    let s = f.get_local(idx)?;
    f.push(s);

    Ok(StepControl::Continue)
}
