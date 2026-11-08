use crate::{
    engines::{
        exec_error::ExecResult,
        interpreter::{interpreter_frame::InterpreterFrame, step_outcome::StepControl},
    },
    oops::constant_entries::{LdC2Loadable, LdCLoadable},
};
use crate::{gc_bindings::oop_hierarchy::NObjPtr, oops::jvalue::*};

pub fn nop<'exec, 'meta: 'exec>(
    _: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    Ok(StepControl::Continue)
}

pub fn aconst_null<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    f.push_ref(NObjPtr::null());
    Ok(StepControl::Continue)
}

pub fn iconst<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    value: JInt,
) -> ExecResult<StepControl<'exec, 'meta>> {
    f.push_int(value.into());
    Ok(StepControl::Continue)
}

pub fn lconst<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    value: JLong,
) -> ExecResult<StepControl<'exec, 'meta>> {
    f.push_long(value.into());
    Ok(StepControl::Continue)
}

pub fn fconst<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    value: JFloat,
) -> ExecResult<StepControl<'exec, 'meta>> {
    f.push_float(value.into());
    Ok(StepControl::Continue)
}

pub fn dconst<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    value: JDouble,
) -> ExecResult<StepControl<'exec, 'meta>> {
    f.push_double(value.into());
    Ok(StepControl::Continue)
}

pub fn bipush<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    value: JByte,
) -> ExecResult<StepControl<'exec, 'meta>> {
    f.push_int(value.into());
    Ok(StepControl::Continue)
}

pub fn sipush<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    value: JShort,
) -> ExecResult<StepControl<'exec, 'meta>> {
    f.push_int(value.into());
    Ok(StepControl::Continue)
}

pub fn ldc<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    loadable: LdCLoadable,
) -> ExecResult<StepControl<'exec, 'meta>> {
    match loadable {
        LdCLoadable::Float(value) => f.push_float(value),
        LdCLoadable::Integer(value) => f.push_int(value),
    }

    Ok(StepControl::Continue)
}

pub fn ldc2<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    loadable: LdC2Loadable,
) -> ExecResult<StepControl<'exec, 'meta>> {
    match loadable {
        LdC2Loadable::Long(l) => f.push_long(l),
        LdC2Loadable::Double(d) => f.push_double(d),
    }

    Ok(StepControl::Continue)
}
