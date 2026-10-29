use crate::{engines::{exec_error::ExecResult, interpreter::{interpreter_frame::InterpreterFrame, step_outcome::StepControl}}, oops::constant_entries::MethodRef};

pub fn invoke_static(f: &InterpreterFrame, mref: &MethodRef) -> ExecResult<StepControl> {
    unimplemented!()
}
