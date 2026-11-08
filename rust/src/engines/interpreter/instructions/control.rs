use crate::engines::protocol::MethodReturn;
use crate::{code::instructions::InstIdx, oops::jvalue::JValue};
use crate::{
    code::instructions::{LookupTable, RangeTable},
    engines::{
        exec_error::{ExecErrorKind, ExecResult},
        interpreter::{interpreter_frame::InterpreterFrame, slot::Slot, step_outcome::StepControl},
    },
};

pub fn ireturn<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let value = f.pop_int()?;
    Ok(StepControl::Return(MethodReturn::Value(JValue::Int(value))))
}

pub fn lreturn<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let value = f.pop_long()?;
    Ok(StepControl::Return(MethodReturn::Value(JValue::Long(
        value,
    ))))
}

pub fn freturn<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let value = f.pop_float()?;
    Ok(StepControl::Return(MethodReturn::Value(JValue::Float(
        value,
    ))))
}

pub fn dreturn<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let value = f.pop_double()?;
    Ok(StepControl::Return(MethodReturn::Value(JValue::Double(
        value,
    ))))
}

pub fn areturn<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let value = f.pop_ref()?;
    Ok(StepControl::Return(MethodReturn::Value(JValue::Ref(value))))
}

pub fn return_void<'exec, 'meta: 'exec>(
    _: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    Ok(StepControl::Return(MethodReturn::Void))
}

pub fn branch<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    target: InstIdx,
    taken: bool,
) -> ExecResult<StepControl<'exec, 'meta>> {
    if taken {
        f.jump_to(target)?;
    }
    Ok(StepControl::Continue)
}

pub fn table_switch<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    rt: &RangeTable,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let Slot::Int(index) = f.pop()? else {
        return Err(f.make_exec_error(ExecErrorKind::MismatchSlotType));
    };
    let target = rt.get_inst_idx(index);

    branch(f, target, true)
}

pub fn lookup_switch<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    lt: &LookupTable,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let Slot::Int(key) = f.pop()? else {
        return Err(f.make_exec_error(ExecErrorKind::MismatchSlotType));
    };
    let target = lt.get_inst_idx(key);

    branch(f, target, true)
}
