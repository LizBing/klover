use crate::engines::{exec_error::ExecResult, invocation::Invocation, protocol::MethodReturn};

pub enum StepControl<'exec, 'meta: 'exec> {
    Continue,
    Return(MethodReturn),
    Invoke(Invocation<'exec, 'meta>),
}

pub struct StepOutcome<'exec, 'meta: 'exec> {
    pub cost: u64,
    pub result: ExecResult<StepControl<'exec, 'meta>>,
}

impl<'exec, 'meta: 'exec> StepOutcome<'exec, 'meta> {
    pub(super) fn new(cost: u64, result: ExecResult<StepControl<'exec, 'meta>>) -> Self {
        Self {
            cost,
            result,
        }
    }
}
