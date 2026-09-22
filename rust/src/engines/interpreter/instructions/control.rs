use crate::{
    code::instructions::InstIdx, engines::exec_dispatcher::MethodReturn, oops::jvalue::JValue,
};
use crate::{
    code::instructions::{LookupTable, RangeTable},
    engines::{
        exec_error::{ExecErrorKind, ExecResult},
        interpreter::{interpreter_frame::InterpreterFrame, slot::Slot, step_outcome::StepControl},
    },
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

pub fn table_switch(f: &mut InterpreterFrame, rt: &RangeTable) -> ExecResult<StepControl> {
    let Slot::Int(index) = f.pop()? else {
        return Err(f.make_exec_error(ExecErrorKind::MismatchSlotType));
    };
    let target = rt.get_inst_idx(index);

    branch(f, target, true)
}

pub fn lookup_switch(f: &mut InterpreterFrame, lt: &LookupTable) -> ExecResult<StepControl> {
    let Slot::Int(key) = f.pop()? else {
        return Err(f.make_exec_error(ExecErrorKind::MismatchSlotType));
    };
    let target = lt.get_inst_idx(key);

    branch(f, target, true)
}
