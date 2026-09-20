use crate::engines::{
    exec_error::ExecResult,
    interpreter::{interpreter_frame::InterpreterFrame, step_outcome::StepControl},
};
use crate::{
    code::instructions::InstIdx, engines::exec_dispatcher::MethodReturn, oops::jvalue::JValue,
};

pub fn ireturn(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let value = f.pop_int()?;
    Ok(StepControl::Return(MethodReturn::Value(JValue::Int(value))))
}

pub fn lreturn(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let value = f.pop_long()?;
    Ok(StepControl::Return(MethodReturn::Value(JValue::Long(
        value,
    ))))
}

pub fn freturn(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let value = f.pop_float()?;
    Ok(StepControl::Return(MethodReturn::Value(JValue::Float(
        value,
    ))))
}

pub fn dreturn(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let value = f.pop_double()?;
    Ok(StepControl::Return(MethodReturn::Value(JValue::Double(
        value,
    ))))
}

pub fn areturn(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let value = f.pop_ref()?;
    Ok(StepControl::Return(MethodReturn::Value(JValue::Ref(value))))
}

pub fn return_void(_: &mut InterpreterFrame) -> ExecResult<StepControl> {
    Ok(StepControl::Return(MethodReturn::Void))
}

pub fn branch(f: &mut InterpreterFrame, target: InstIdx, taken: bool) -> ExecResult<StepControl> {
    if taken {
        f.jump_to(target)?;
    }
    Ok(StepControl::Continue)
}
