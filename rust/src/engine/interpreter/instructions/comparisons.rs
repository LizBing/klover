use crate::engine::{
    exec_error::ExecResult, interpreter::interpreter_frame::InterpreterFrame, outcome::StepOutcome,
};

fn branch_if(f: &mut InterpreterFrame, condition: bool) -> ExecResult<StepOutcome> {
    let offset = f.read_i16()?;

    if condition {
        Ok(StepOutcome::Branch(f.branch_target(offset)?))
    } else {
        Ok(StepOutcome::Continue)
    }
}

fn if_zero(
    f: &mut InterpreterFrame,
    predicate: impl FnOnce(i32) -> bool,
) -> ExecResult<StepOutcome> {
    let value = f.pop()?.as_int()?;
    branch_if(f, predicate(value))
}

fn if_icmp(
    f: &mut InterpreterFrame,
    predicate: impl FnOnce(i32, i32) -> bool,
) -> ExecResult<StepOutcome> {
    let rhs = f.pop()?.as_int()?;
    let lhs = f.pop()?.as_int()?;
    branch_if(f, predicate(lhs, rhs))
}

pub fn ifeq(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    if_zero(f, |value| value == 0)
}

pub fn ifne(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    if_zero(f, |value| value != 0)
}

pub fn iflt(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    if_zero(f, |value| value < 0)
}

pub fn ifge(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    if_zero(f, |value| value >= 0)
}

pub fn ifgt(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    if_zero(f, |value| value > 0)
}

pub fn ifle(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    if_zero(f, |value| value <= 0)
}

pub fn if_icmpeq(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    if_icmp(f, |lhs, rhs| lhs == rhs)
}

pub fn if_icmpne(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    if_icmp(f, |lhs, rhs| lhs != rhs)
}

pub fn if_icmplt(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    if_icmp(f, |lhs, rhs| lhs < rhs)
}

pub fn if_icmpge(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    if_icmp(f, |lhs, rhs| lhs >= rhs)
}

pub fn if_icmpgt(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    if_icmp(f, |lhs, rhs| lhs > rhs)
}

pub fn if_icmple(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    if_icmp(f, |lhs, rhs| lhs <= rhs)
}

pub fn goto(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
    let offset = f.read_i16()?;
    Ok(StepOutcome::Branch(f.branch_target(offset)?))
}
