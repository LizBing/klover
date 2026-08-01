use crate::{
    engine::{
        exec_error::{ExecError, ExecResult},
        interpreter::interpreter_frame::InterpreterFrame,
        outcome::StepOutcome,
        resolved_method::ResolvedMethod,
    },
    oops::acc_flags::AccFlags,
};

/// Resolve the method reference and hand the actual frame transition to the
/// dispatcher.  The dispatcher owns the JavaThread, so it is the only layer
/// that can perform class initialization immediately before entering the
/// target frame.
pub fn invokestatic(frame: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let index = frame.read_u16()? as usize;
    let resolved = frame.resolve_method_ref(index)?;

    if !resolved.method.acc_flags.contains(AccFlags::ACC_STATIC) {
        return Err(ExecError::IncompatibleStaticCall);
    }

    let arg_slots = resolved.method.desc.parameter_slot_count();

    Ok(StepOutcome::InvokeStatic {
        target: ResolvedMethod::from(resolved),
        arg_slots,
    })
}
