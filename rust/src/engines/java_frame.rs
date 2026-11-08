use crate::engines::{
    exec_error::ExecResult,
    protocol::{EngineExit, ExecBudget, MethodReturn},
};

pub trait JavaFrame<'exec, 'meta: 'exec> {
    fn resume(&mut self, budget: &mut ExecBudget) -> ExecResult<EngineExit<'exec, 'meta>>;

    fn accept_return(&mut self, value: MethodReturn) -> ExecResult<()>;

    // fn try_handle_exception()
}
