use crate::{
    engines::{
        exec_error::{ExecErrorKind, ExecResult},
        interpreter::{interpreter_frame::InterpreterFrame, slot::Slot, step_outcome::StepControl},
        invocation::{CallTarget, Invocation},
    },
    oops::{constant_entries::MethodRef, jvalue::JValue, method::MethodAccessFlags},
};

/// Prepare an interpreted static call. The dispatcher owns callee execution.
pub fn invoke_static<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    mref: &MethodRef<'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let resolved = mref
        .resolve(f.target.declaring_class)
        .map_err(|e| f.make_exec_error(ExecErrorKind::Resolve(e)))?;
    let target = CallTarget::from(resolved);
    let method = target.method;
    let unsupported = |reason| {
        f.make_exec_error(ExecErrorKind::UnsupportedInvocation {
            owner: target.declaring_class.name().utf8().into(),
            name: method.name.utf8().into(),
            desc: method.desc.raw().utf8().into(),
            reason,
        })
    };
    if matches!(method.name.utf8(), "<init>" | "<clinit>") {
        return Err(unsupported(
            "initialization methods cannot be called by invokestatic",
        ));
    }
    if !method.acc_flags.contains(MethodAccessFlags::STATIC) {
        return Err(f.make_exec_error(ExecErrorKind::ExpectedStaticMethod {
            owner: target.declaring_class.name().utf8().into(),
            name: method.name.utf8().into(),
            desc: method.desc.raw().utf8().into(),
        }));
    }
    if method.acc_flags.contains(MethodAccessFlags::NATIVE) {
        return Err(unsupported("native methods are unsupported"));
    }
    if method.acc_flags.contains(MethodAccessFlags::SYNCHRONIZED) {
        return Err(unsupported("synchronized methods are unsupported"));
    }
    if method.code.is_none() {
        return Err(f.make_exec_error(ExecErrorKind::NoCode {
            owner: target.declaring_class.name().utf8().into(),
            name: method.name.utf8().into(),
            desc: method.desc.raw().utf8().into(),
        }));
    }
    if let Some(klass) = target.declaring_class.required_initializer() {
        return Err(
            f.make_exec_error(ExecErrorKind::UnsupportedClassInitialization {
                owner: klass.name().utf8().into(),
            }),
        );
    }

    // Read the entire argument suffix before mutating it. A failed resolution,
    // access check, or argument check leaves the caller's stack intact.
    let stack = f.operand_stack();
    let mut end = stack.len();
    let mut args = Vec::with_capacity(method.desc.args().len());
    for parameter in method.desc.args().iter().rev() {
        let width = if matches!(parameter.raw().utf8(), "J" | "D") {
            2
        } else {
            1
        };
        let start = end
            .checked_sub(width)
            .ok_or_else(|| f.make_exec_error(ExecErrorKind::StackUnderflow))?;
        let value = match (parameter.raw().utf8().as_bytes()[0], &stack[start..end]) {
            (b'Z' | b'B' | b'C' | b'S' | b'I', [Slot::Int(value)]) => JValue::Int(*value),
            (b'F', [Slot::Float(value)]) => JValue::Float(*value),
            (b'L' | b'[', [Slot::Ref(value)]) => JValue::Ref(*value),
            (b'J', [Slot::LongHigh(high), Slot::LongLow(low)]) => {
                JValue::Long(i64::from_be_bytes([
                    high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3],
                ]))
            }
            (b'D', [Slot::DoubleHigh(high), Slot::DoubleLow(low)]) => {
                JValue::Double(f64::from_be_bytes([
                    high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3],
                ]))
            }
            _ => return Err(f.make_exec_error(ExecErrorKind::MismatchSlotType)),
        };
        args.push(value);
        end = start;
    }
    args.reverse();
    let invocation = Invocation::try_new_bytecode(target, args.into_boxed_slice())
        .map_err(|error| f.make_exec_error(error.kind))?;
    let consumed = stack.len() - end;
    f.replace_stack_top(consumed, &[]);
    Ok(StepControl::Invoke(invocation))
}

#[cfg(test)]
#[path = "../../../tests/invoke_static.rs"]
mod tests;
