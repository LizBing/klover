use crate::engines::{
    exec_error::{ExecErrorKind, ExecResult},
    interpreter::{interpreter_frame::InterpreterFrame, slot::Slot, step_outcome::StepControl},
};

// Consume a group of one or two slots, without splitting a category-2 value.
// Returned slices retain their original bottom-to-top order.
fn group<'a>(
    stack: &'a [Slot],
    end: &mut usize,
    width: usize,
) -> Result<&'a [Slot], ExecErrorKind> {
    let start = end
        .checked_sub(width)
        .ok_or(ExecErrorKind::StackUnderflow)?;
    let slots = &stack[start..*end];
    let category1 = |slot: &Slot| matches!(slot, Slot::Int(_) | Slot::Float(_) | Slot::Ref(_));
    let valid = match slots {
        [one] => category1(one),
        [Slot::LongHigh(_), Slot::LongLow(_)] | [Slot::DoubleHigh(_), Slot::DoubleLow(_)] => true,
        [one, two] => category1(one) && category1(two),
        _ => false,
    };
    if !valid {
        return Err(ExecErrorKind::MismatchSlotType);
    }
    *end = start;
    Ok(slots)
}

fn discard(f: &mut InterpreterFrame, width: usize) -> ExecResult<StepControl> {
    let mut end = f.operand_stack().len();
    group(f.operand_stack(), &mut end, width).map_err(|e| f.make_exec_error(e))?;
    f.replace_stack_top(width, &[]);
    Ok(StepControl::Continue)
}

// All dup forms duplicate a one- or two-slot group and insert it beneath
// another whole group. Validate both groups before mutating the stack.
fn duplicate(f: &mut InterpreterFrame, width: usize, beneath: usize) -> ExecResult<StepControl> {
    let stack = f.operand_stack();
    let mut end = stack.len();
    let top = group(stack, &mut end, width).map_err(|e| f.make_exec_error(e))?;
    let below = if beneath == 0 {
        &[][..]
    } else {
        group(stack, &mut end, beneath).map_err(|e| f.make_exec_error(e))?
    };
    // The largest form, dup2_x2, produces six slots.
    let mut replacement = [Slot::Unused; 6];
    let size = width * 2 + beneath;
    replacement[..width].copy_from_slice(top);
    replacement[width..width + beneath].copy_from_slice(below);
    replacement[width + beneath..size].copy_from_slice(top);
    f.replace_stack_top(width + beneath, &replacement[..size]);
    Ok(StepControl::Continue)
}

pub fn pop(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    discard(f, 1)
}
pub fn pop2(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    discard(f, 2)
}
pub fn dup(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    duplicate(f, 1, 0)
}
pub fn dup_x1(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    duplicate(f, 1, 1)
}
pub fn dup_x2(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    duplicate(f, 1, 2)
}
pub fn dup2(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    duplicate(f, 2, 0)
}
pub fn dup2_x1(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    duplicate(f, 2, 1)
}
pub fn dup2_x2(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    duplicate(f, 2, 2)
}

pub fn swap(f: &mut InterpreterFrame) -> ExecResult<StepControl> {
    let stack = f.operand_stack();
    let mut end = stack.len();
    let top = group(stack, &mut end, 1).map_err(|e| f.make_exec_error(e))?[0];
    let below = group(stack, &mut end, 1).map_err(|e| f.make_exec_error(e))?[0];
    f.replace_stack_top(2, &[top, below]);
    Ok(StepControl::Continue)
}
