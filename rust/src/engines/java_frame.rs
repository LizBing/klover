use crate::engines::{exec_dispatcher::{EngineExit, ExecBudget, MethodReturn}, exec_error::ExecResult};

pub trait JavaFrame {
    fn resume(
        &mut self,
        budget: &mut ExecBudget,
    ) -> ExecResult<EngineExit>;

    fn accept_return(
        &mut self,
        value: MethodReturn,
    ) -> ExecResult<()>;

    // fn try_handle_exception()
}
