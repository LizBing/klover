use crate::engine::{engine_error::ExecResult, interpreter::interpreter_frame::InterpreterFrame, outcome::StepOutcome, slot::Slot};

pub fn iadd(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let value2 = f.pop()?.as_int()?;
    let value1 = f.pop()?.as_int()?;
    
    let result = value1 + value2;

    f.push(Slot::int(result))?;

    Ok(StepOutcome::Continue)
}
