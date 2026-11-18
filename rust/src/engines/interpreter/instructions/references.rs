use crate::{
    engines::{
        exec_error::{ExecErrorKind, ExecResult},
        interpreter::{interpreter_frame::InterpreterFrame, slot::Slot, step_outcome::StepControl},
        invocation::{CallTarget, Invocation},
    },
    oops::{
        constant_entries::MethodRef,
        jvalue::{JBoolean, JByte, JChar, JDouble, JInt, JLong, JShort, JValue},
    },
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
    if !method.is_static() {
        return Err(f.make_exec_error(ExecErrorKind::ExpectedStaticMethod {
            owner: target.declaring_class.name().utf8().into(),
            name: method.name.utf8().into(),
            desc: method.desc.raw().utf8().into(),
        }));
    }
    if method.is_native() {
        return Err(unsupported("native methods are unsupported"));
    }
    if method.is_synchronized() {
        return Err(unsupported("synchronized methods are unsupported"));
    }
    if method.code.is_none() {
        return Err(f.make_exec_error(ExecErrorKind::NoCode {
            owner: target.declaring_class.name().utf8().into(),
            name: method.name.utf8().into(),
            desc: method.desc.raw().utf8().into(),
        }));
    }
    if !target.declaring_class.initialization_ready(f.thread_id) {
        return Ok(StepControl::Initialize(target.declaring_class));
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
                JValue::Long(JLong(i64::from_be_bytes([
                    high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3],
                ])))
            }
            (b'D', [Slot::DoubleHigh(high), Slot::DoubleLow(low)]) => {
                JValue::Double(JDouble(f64::from_be_bytes([
                    high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3],
                ])))
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

fn resolve_static<'meta>(
    f: &InterpreterFrame<'_, 'meta>,
    fref: &crate::oops::constant_entries::FieldRef<'meta>,
    writing: bool,
) -> ExecResult<crate::oops::constant_entries::ResolvedField<'meta>> {
    let resolved = fref
        .resolve(f.target.declaring_class)
        .map_err(|e| f.make_exec_error(ExecErrorKind::Resolve(e)))?;
    let field = resolved.field;
    let owner = resolved.declaring_klass;
    if !field.is_static() {
        return Err(f.make_exec_error(ExecErrorKind::ExpectedStaticField {
            owner: owner.name().utf8().into(),
            name: field.name.utf8().into(),
            desc: field.desc.raw().utf8().into(),
        }));
    }
    if writing
        && field.is_final()
        && !(std::ptr::eq(owner, f.target.declaring_class)
            && f.target.method.name.utf8() == "<clinit>")
    {
        return Err(f.make_exec_error(ExecErrorKind::IllegalFinalFieldWrite {
            owner: owner.name().utf8().into(),
            name: field.name.utf8().into(),
            desc: field.desc.raw().utf8().into(),
        }));
    }
    Ok(resolved)
}

pub fn get_static<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    fref: &crate::oops::constant_entries::FieldRef<'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let resolved = resolve_static(f, fref, false)?;
    if !resolved.declaring_klass.initialization_ready(f.thread_id) {
        return Ok(StepControl::Initialize(resolved.declaring_klass));
    }
    f.push_value(resolved.load_static());
    Ok(StepControl::Continue)
}

pub fn put_static<'exec, 'meta: 'exec>(
    f: &mut InterpreterFrame<'exec, 'meta>,
    fref: &crate::oops::constant_entries::FieldRef<'meta>,
) -> ExecResult<StepControl<'exec, 'meta>> {
    let resolved = resolve_static(f, fref, true)?;
    if !resolved.declaring_klass.initialization_ready(f.thread_id) {
        return Ok(StepControl::Initialize(resolved.declaring_klass));
    }
    let field = resolved.field;
    let descriptor = field.desc.raw().utf8().as_bytes()[0];
    let width = if matches!(descriptor, b'J' | b'D') {
        2
    } else {
        1
    };
    let start = f
        .operand_stack()
        .len()
        .checked_sub(width)
        .ok_or_else(|| f.make_exec_error(ExecErrorKind::StackUnderflow))?;
    // Validate the entire suffix before modifying the field or operand stack.
    let value = match (descriptor, &f.operand_stack()[start..]) {
        (b'Z', [Slot::Int(JInt(v))]) => JValue::Boolean(JBoolean((*v & 1) as u8)),
        (b'B', [Slot::Int(JInt(v))]) => JValue::Byte(JByte(*v as i8)),
        (b'C', [Slot::Int(JInt(v))]) => JValue::Char(JChar(*v as u16)),
        (b'S', [Slot::Int(JInt(v))]) => JValue::Short(JShort(*v as i16)),
        (b'I', [Slot::Int(v)]) => JValue::Int(*v),
        (b'F', [Slot::Float(v)]) => JValue::Float(*v),
        (b'J', [Slot::LongHigh(high), Slot::LongLow(low)]) => {
            JValue::Long(JLong(i64::from_be_bytes([
                high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3],
            ])))
        }
        (b'D', [Slot::DoubleHigh(high), Slot::DoubleLow(low)]) => {
            JValue::Double(JDouble(f64::from_be_bytes([
                high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3],
            ])))
        }
        (b'L' | b'[', [Slot::Ref(v)]) => JValue::Ref(*v),
        _ => return Err(f.make_exec_error(ExecErrorKind::MismatchSlotType)),
    };
    resolved.store_static(value);
    f.replace_stack_top(width, &[]);
    Ok(StepControl::Continue)
}

#[cfg(test)]
#[path = "../../../tests/static_fields.rs"]
mod static_fields_tests;
