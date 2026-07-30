use crate::engine::{
    engine_error::ExecResult, interpreter::interpreter_frame::InterpreterFrame,
    outcome::StepOutcome,
};

fn istore_at(f: &mut InterpreterFrame, index: usize) -> ExecResult<StepOutcome> {
    let value = f.pop()?;
    value.as_int()?;
    f.set_local(index, value)?;
    Ok(StepOutcome::Continue)
}

pub fn istore(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u8()? as usize;
    istore_at(f, index)
}

pub fn istore_n<const N: usize>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    istore_at(f, N)
}
