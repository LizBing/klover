use crate::engine::{resolved_method::ResolvedMethod, slot::Slot};

#[derive(Debug)]
pub struct Invocation {
    pub target: ResolvedMethod,
    pub args: Vec<Slot>,
}
