use crate::{engine::{engine_error::{ExecError, ExecResult}, interpreter::instructions::{control::ireturn, loads::iload_n, math::iadd}, outcome::StepOutcome}, runtime::java_thread::JavaThread};

#[derive(Debug)]
pub struct Interpreter;

impl Interpreter {
    pub fn execute_one(&mut self, thrd: &mut JavaThread) -> ExecResult<StepOutcome> {
        let frame = thrd
            .stack_mut()
            .current_interpreter_mut()
            .map_err(|e| ExecError::Stack(e))?;

        let opc = frame.fetch_opcode()?;

        match opc {
            26 => iload_n::<0>(frame),
            27 => iload_n::<1>(frame),
            96 => iadd(frame),
            172 => ireturn(frame),

            unsupported => Err(ExecError::UnsupportedOpcode {
                opcode: unsupported,
                bci: frame.last_pc()
            })
        }
    }
}
