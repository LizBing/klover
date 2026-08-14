use crate::{engines::{exec_error::ExecResult, interpreter::{interpreter_frame::InterpreterFrame, step_outcome::StepOutcome}, slot::Slot}, gc_bindings::oop_hierarchy::NObjPtr, oops::jvalue::JInt};

// 0x00
pub fn nop(_: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    Ok(StepOutcome::Continue)
}

// 0x01
pub fn aconst_null(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push(Slot::Ref(NObjPtr::null()));

    Ok(StepOutcome::Continue)
}

// 0x02..=0x08
pub fn iconst_n<const N: i32>(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push(Slot::Int(N));

    Ok(StepOutcome::Continue)
}

// 0x09
pub fn lconst_0(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push_long(0);

    Ok(StepOutcome::Continue)
}

// 0x0a
pub fn lconst_1(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push_long(1);

    Ok(StepOutcome::Continue)
}

// 0x0b
pub fn fconst_0(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push(Slot::Float(0.0));

    Ok(StepOutcome::Continue)
}

// 0x0c
pub fn fconst_1(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push(Slot::Float(1.0));

    Ok(StepOutcome::Continue)
}

// 0x0d
pub fn fconst_2(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push(Slot::Float(2.0));

    Ok(StepOutcome::Continue)
}

// 0x0e
pub fn dconst_0(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push_double(0.0);

    Ok(StepOutcome::Continue)
}

// 0x0f
pub fn dconst_1(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    f.push_double(1.0);

    Ok(StepOutcome::Continue)
}

// 0x10
pub fn bipush(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let byte = f.read_jbyte()?;
    f.push(Slot::Int(byte as JInt));

    Ok(StepOutcome::Continue)
}

// 0x11
pub fn sipush(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let short = f.read_jshort()?;
    f.push(Slot::Int(short as JInt));

    Ok(StepOutcome::Continue)
}


