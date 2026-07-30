use crate::engine::{engine_error::ExecResult, interpreter::interpreter_frame::InterpreterFrame, outcome::{RetValue, StepOutcome}};

pub fn ireturn(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let value = f.pop()?.as_int()?;

    Ok(StepOutcome::Return(RetValue::Int(value)))
}
