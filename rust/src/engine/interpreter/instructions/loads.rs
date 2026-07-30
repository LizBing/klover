use crate::engine::{
    engine_error::{ExecResult},
    interpreter::interpreter_frame::InterpreterFrame,
    outcome::StepOutcome,
    slot::Slot,
};

fn iload_at(f: &mut InterpreterFrame, index: usize) -> ExecResult<StepOutcome> {
    let value = f.get_local(index)?;
    value.as_int()?;
    f.push(value)?;
    Ok(StepOutcome::Continue)
}

fn lload_at(f: &mut InterpreterFrame, index: usize) -> ExecResult<StepOutcome> {
    let high = f.get_local(index)?;
    let low = f.get_local(index + 1)?;
    f.push_long(Slot::as_long(high, low)?)?;
    Ok(StepOutcome::Continue)
}

fn fload_at(f: &mut InterpreterFrame, index: usize) -> ExecResult<StepOutcome> {
    let value = f.get_local(index)?;
    value.as_float()?;
    f.push(value)?;
    Ok(StepOutcome::Continue)
}

fn dload_at(f: &mut InterpreterFrame, index: usize) -> ExecResult<StepOutcome> {
    let high = f.get_local(index)?;
    let low = f.get_local(index + 1)?;
    f.push_double(Slot::as_double(high, low)?)?;
    Ok(StepOutcome::Continue)
}

pub fn iload(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u8()? as usize;
    iload_at(f, index)
}

pub fn lload(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u8()? as usize;
    lload_at(f, index)
}

pub fn fload(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u8()? as usize;
    fload_at(f, index)
}

pub fn dload(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u8()? as usize;
    dload_at(f, index)
}

pub fn iload_n<const N: usize>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    iload_at(f, N)
}

pub fn lload_n<const N: usize>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    lload_at(f, N)
}

pub fn fload_n<const N: usize>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    fload_at(f, N)
}

pub fn dload_n<const N: usize>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    dload_at(f, N)
}
