use std::marker::PhantomData;

use crate::{engines::{exec_dispatcher::MethodReturn, exec_error::ExecResult}, oops::jvalue::JValue};

pub enum StepControl {
    Continue,
    Return(MethodReturn),
}

pub struct StepOutcome {
    __: PhantomData<()>,

    pub cost: u64,
    pub result: ExecResult<StepControl>,
}

impl StepOutcome {
    pub(super) fn new(cost: u64, result: ExecResult<StepControl>) -> Self {
        Self {
            __: PhantomData,
            cost,
            result,
        }
    }
}
