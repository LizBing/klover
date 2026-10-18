use crate::{engines::{
    exec_error::ExecResult,
    interpreter::{interpreter_frame::InterpreterFrame, step_outcome::StepControl},
}, oops::constant_entries::{LdC2Loadable, LdCLoadable}};
use crate::{gc_bindings::oop_hierarchy::NObjPtr, oops::jvalue::*};

pub fn nop(_: &mut InterpreterFrame) -> ExecResult<StepControl> {
    Ok(StepControl::Continue)
}

pub fn aconst_null(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    f.push_ref(NObjPtr::null());
    Ok(StepControl::Continue)
}

pub fn iconst(f: &mut InterpreterFrame, value: JInt) -> ExecResult<StepControl> {
    f.push_int(value.into());
    Ok(StepControl::Continue)
}

pub fn lconst(f: &mut InterpreterFrame, value: JLong) -> ExecResult<StepControl> {
    f.push_long(value.into());
    Ok(StepControl::Continue)
}

pub fn fconst(f: &mut InterpreterFrame, value: JFloat) -> ExecResult<StepControl> {
    f.push_float(value.into());
    Ok(StepControl::Continue)
}

pub fn dconst(f: &mut InterpreterFrame, value: JDouble) -> ExecResult<StepControl> {
    f.push_double(value.into());
    Ok(StepControl::Continue)
}

pub fn bipush(f: &mut InterpreterFrame, value: JByte) -> ExecResult<StepControl> {
    f.push_int(value.into());
    Ok(StepControl::Continue)
}

pub fn sipush(f: &mut InterpreterFrame, value: JShort) -> ExecResult<StepControl> {
    f.push_int(value.into());
    Ok(StepControl::Continue)
}

pub fn ldc(f: &mut InterpreterFrame, loadable: LdCLoadable) -> ExecResult<StepControl> {
    match loadable {
        LdCLoadable::Float(value) => f.push_float(value),
        LdCLoadable::Integer(value) => f.push_int(value),
    }

    Ok(StepControl::Continue)
}

pub fn ldc2(f: &mut InterpreterFrame, loadable: LdC2Loadable) -> ExecResult<StepControl> {
    match loadable {
        LdC2Loadable::Long(l) => f.push_long(l),
        LdC2Loadable::Double(d) => f.push_double(d),
    }

    Ok(StepControl::Continue)
}
