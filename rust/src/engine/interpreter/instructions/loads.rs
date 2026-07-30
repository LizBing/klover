use crate::engine::{engine_error::ExecResult, interpreter::interpreter_frame::InterpreterFrame, outcome::StepOutcome};

pub fn iload_n<const N: usize>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let s = f.get_local(N)?;
    s.as_int()?;
    f.push(s)?;

    Ok(StepOutcome::Continue)
}
