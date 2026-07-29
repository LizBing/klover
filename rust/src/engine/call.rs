use crate::engine::{interpreter::interpreter_frame::ResolvedMethod, slot::Slot};

#[derive(Debug)]
pub struct Invocation {
    pub target: ResolvedMethod,
    pub args: Vec<Slot>,
}
