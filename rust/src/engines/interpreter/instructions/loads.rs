use crate::engines::{exec_error::ExecResult, interpreter::{interpreter_frame::InterpreterFrame, step_outcome::StepOutcome}};

// 0x1a..=0x1d
pub fn iload_n<const N: usize>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let s = f.get_local(N)?;
    f.push(s);

    Ok(StepOutcome::Continue)
}
