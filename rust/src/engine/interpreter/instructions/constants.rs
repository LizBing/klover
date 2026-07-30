use crate::engine::{engine_error::{ExecError, ExecResult}, interpreter::interpreter_frame::InterpreterFrame, outcome::StepOutcome, slot::Slot};

pub fn iconst_n<const N: i32>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push(Slot::int(N))?;
    Ok(StepOutcome::Continue)
}
