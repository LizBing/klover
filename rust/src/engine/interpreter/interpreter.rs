use crate::{engine::{engine_error::{ExecError, ExecResult}, outcome::StepOutcome}, runtime::java_thread::JavaThread};

#[derive(Debug)]
pub struct Interpreter;

impl Interpreter {
    pub fn execute_one(&mut self, thrd: &mut JavaThread) -> ExecResult<StepOutcome> {
        unimplemented!()
    }
}
