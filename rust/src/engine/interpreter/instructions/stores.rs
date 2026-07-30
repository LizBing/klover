use crate::engine::{
    engine_error::{ExecError, ExecResult},
    interpreter::interpreter_frame::InterpreterFrame,
    outcome::StepOutcome,
    slot::Slot,
};

fn ensure_local_width(f: &InterpreterFrame, index: usize, width: usize) -> ExecResult<()> {
    let last_offset = width
        .checked_sub(1)
        .ok_or(ExecError::InvalidLocalIndex(index))?;
    let last_index = index
        .checked_add(last_offset)
        .ok_or(ExecError::InvalidLocalIndex(index))?;
    f.get_local(last_index)?;
    Ok(())
}

fn istore_at(f: &mut InterpreterFrame, index: usize) -> ExecResult<StepOutcome> {
    ensure_local_width(f, index, 1)?;
    let value = f.pop()?;
    value.as_int()?;
    f.set_local(index, value)?;
    Ok(StepOutcome::Continue)
}

fn lstore_at(f: &mut InterpreterFrame, index: usize) -> ExecResult<StepOutcome> {
    ensure_local_width(f, index, 2)?;
    let value = f.pop_long()?;
    f.set_local(index, Slot::long_high(value))?;
    f.set_local(index + 1, Slot::long_low(value))?;
    Ok(StepOutcome::Continue)
}

fn fstore_at(f: &mut InterpreterFrame, index: usize) -> ExecResult<StepOutcome> {
    ensure_local_width(f, index, 1)?;
    let value = f.pop()?;
    value.as_float()?;
    f.set_local(index, value)?;
    Ok(StepOutcome::Continue)
}

fn dstore_at(f: &mut InterpreterFrame, index: usize) -> ExecResult<StepOutcome> {
    ensure_local_width(f, index, 2)?;
    let value = f.pop_double()?;
    f.set_local(index, Slot::double_high(value))?;
    f.set_local(index + 1, Slot::double_low(value))?;
    Ok(StepOutcome::Continue)
}

fn astore_at(f: &mut InterpreterFrame, index: usize) -> ExecResult<StepOutcome> {
    ensure_local_width(f, index, 1)?;
    let value = f.pop()?;
    value.as_ref()?;
    f.set_local(index, value)?;
    Ok(StepOutcome::Continue)
}

pub fn istore(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u8()? as usize;
    istore_at(f, index)
}

pub fn lstore(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u8()? as usize;
    lstore_at(f, index)
}

pub fn fstore(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u8()? as usize;
    fstore_at(f, index)
}

pub fn dstore(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u8()? as usize;
    dstore_at(f, index)
}

pub fn astore(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u8()? as usize;
    astore_at(f, index)
}

pub fn istore_n<const N: usize>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    istore_at(f, N)
}

pub fn lstore_n<const N: usize>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    lstore_at(f, N)
}

pub fn fstore_n<const N: usize>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    fstore_at(f, N)
}

pub fn dstore_n<const N: usize>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    dstore_at(f, N)
}

pub fn astore_n<const N: usize>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    astore_at(f, N)
}
