use crate::engines::{
    exec_error::ExecResult,
    protocol::{EngineExit, ExecBudget, ExecutionContext, MethodReturn},
};

pub trait JavaFrame<'exec, 'meta: 'exec> {
    fn resume(
        &mut self,
        budget: &mut ExecBudget,
        context: &ExecutionContext,
    ) -> ExecResult<EngineExit<'exec, 'meta>>;

    fn abort(&mut self, _reason: &str) {}

    fn accept_return(&mut self, value: MethodReturn) -> ExecResult<()>;

    // fn try_handle_exception()
}
