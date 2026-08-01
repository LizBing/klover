use crate::engine::{
    exec_error::ExecResult,
    interpreter::interpreter_frame::InterpreterFrame,
    outcome::{RetValue, StepOutcome},
};

pub fn ireturn(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    Ok(StepOutcome::Return(RetValue::Int(f.pop()?.as_int()?)))
}

pub fn lreturn(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    Ok(StepOutcome::Return(RetValue::Long(f.pop_long()?)))
}

pub fn freturn(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    Ok(StepOutcome::Return(RetValue::Float(f.pop()?.as_float()?)))
}

pub fn dreturn(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    Ok(StepOutcome::Return(RetValue::Double(f.pop_double()?)))
}

pub fn areturn(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    Ok(StepOutcome::Return(RetValue::Ref(f.pop()?.as_ref()?)))
}

pub fn return_void(_: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    Ok(StepOutcome::Return(RetValue::Void))
}
