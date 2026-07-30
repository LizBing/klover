use crate::{
    engine::{
        engine_error::{ExecError, ExecResult},
        interpreter::interpreter_frame::InterpreterFrame,
        outcome::StepOutcome,
        slot::Slot,
    },
    oops::cp_entry::CPEntry,
};

enum NumericConstant {
    Int(i32),
    Float(f32),
    Long(i64),
    Double(f64),
}

pub fn nop(_: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    Ok(StepOutcome::Continue)
}

pub fn aconst_null(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push(Slot::reference(0))?;
    Ok(StepOutcome::Continue)
}

pub fn iconst_n<const N: i32>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push(Slot::int(N))?;
    Ok(StepOutcome::Continue)
}

pub fn lconst_n<const N: i64>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push_long(N)?;
    Ok(StepOutcome::Continue)
}

pub fn fconst_n<const N: u32>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push(Slot::float(N as f32))?;
    Ok(StepOutcome::Continue)
}

pub fn dconst_n<const N: u64>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push_double(N as f64)?;
    Ok(StepOutcome::Continue)
}

pub fn bipush(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let value = f.read_i8()? as i32;
    f.push(Slot::int(value))?;
    Ok(StepOutcome::Continue)
}

pub fn sipush(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let value = f.read_i16()? as i32;
    f.push(Slot::int(value))?;
    Ok(StepOutcome::Continue)
}

fn read_numeric_constant(
    f: &InterpreterFrame,
    index: usize,
    wide: bool,
) -> ExecResult<NumericConstant> {
    let entry = f
        .constant_pool_entry(index)
        .ok_or(ExecError::InvalidConstantPoolIndex(index))?;

    match (wide, entry) {
        (false, CPEntry::Integer(value)) => Ok(NumericConstant::Int(*value)),
        (false, CPEntry::Float(value)) => Ok(NumericConstant::Float(*value)),
        (true, CPEntry::Long(value)) => Ok(NumericConstant::Long(*value)),
        (true, CPEntry::Double(value)) => Ok(NumericConstant::Double(*value)),
        (false, CPEntry::Class(_) | CPEntry::StringConstant(_)) => {
            Err(ExecError::UnsupportedLdcConstant { index })
        }
        _ => Err(ExecError::InvalidLdcConstant { index }),
    }
}

fn push_numeric_constant(
    f: &mut InterpreterFrame,
    index: usize,
    wide: bool,
) -> ExecResult<StepOutcome> {
    let constant = read_numeric_constant(f, index, wide)?;

    match constant {
        NumericConstant::Int(value) => f.push(Slot::int(value))?,
        NumericConstant::Float(value) => f.push(Slot::float(value))?,
        NumericConstant::Long(value) => f.push_long(value)?,
        NumericConstant::Double(value) => f.push_double(value)?,
    }

    Ok(StepOutcome::Continue)
}

pub fn ldc(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u8()? as usize;
    push_numeric_constant(f, index, false)
}

pub fn ldc_w(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u16()? as usize;
    push_numeric_constant(f, index, false)
}

pub fn ldc2_w(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = f.read_u16()? as usize;
    push_numeric_constant(f, index, true)
}
