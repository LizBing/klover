use crate::engines::{exec_error::{ExecError::UnsupportedInstruction, ExecResult}, interpreter::{instructions::{control::ireturn, loads::iload_n, math::iadd}, interpreter_frame::InterpreterFrame, step_outcome::StepOutcome}};

pub struct Interpreter;

impl Interpreter {
    pub fn execute_one(f: &mut InterpreterFrame) -> ExecResult<StepOutcome> {
        match f.read_u8()? {
            0x1a => iload_n::<0>(f),
            0x1b => iload_n::<1>(f),

            0x60 => iadd(f),

            0xac => ireturn(f),

            _ => Err(UnsupportedInstruction)
        }
    }
}
