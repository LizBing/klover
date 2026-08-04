use crate::{
    engine::{
        exec_error::{ExecError, ExecResult},
        interpreter::interpreter_frame::InterpreterFrame,
        outcome::StepOutcome,
    },
    oops::{acc_flags::AccFlags, cp_entry::ResolvedFieldRef},
};

fn resolve_static_field(frame: &mut InterpreterFrame) -> ExecResult<ResolvedFieldRef> {
    let index = frame.read_u16()? as usize;
    let resolved = frame.resolve_field_ref(index)?;

    if !resolved.field.acc_flags.contains(AccFlags::ACC_STATIC) {
        return Err(ExecError::IncompatibleStaticFieldAccess);
    }

    Ok(resolved)
}

pub fn getstatic(frame: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    Ok(StepOutcome::GetStatic(resolve_static_field(frame)?))
}

pub fn putstatic(frame: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    Ok(StepOutcome::PutStatic(resolve_static_field(frame)?))
}
