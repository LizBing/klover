use crate::engine::{resolved_method::ResolvedMethod, slot::Slot};

/// A method invocation whose arguments have already been materialized.
///
/// Root entry receives this form directly. An internal bytecode call creates
/// it only after copying arguments from the caller's operand stack.
#[derive(Debug)]
pub struct Invocation {
    pub target: ResolvedMethod,
    pub args: Vec<Slot>,
}
