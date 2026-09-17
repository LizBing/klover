use super::instructions::{control::*, loads::*, math::*};
use crate::code::instructions::Instruction::*;
use crate::engines::exec_dispatcher::{EngineExit, ExecBudget, MethodReturn};
use crate::engines::interpreter::step_outcome::StepControl;
use crate::{
    code::instructions::LocalIdx,
    engines::{
        exec_error::{ExecErrorKind, ExecResult},
        interpreter::{
            instructions::loads, interpreter_frame::InterpreterFrame, step_outcome::StepOutcome,
        },
    },
};

pub struct Interpreter;

impl Interpreter {
    pub fn execute(
        frame: &mut InterpreterFrame,
        budget: &mut ExecBudget,
    ) -> ExecResult<EngineExit> {
        while budget.is_startable() {
            let StepOutcome { cost, result, .. } = Self::execute_one(frame);

            budget.charge(cost);

            match result? {
                StepControl::Continue => {},
                StepControl::Return(value) => {
                    return Ok(EngineExit::Return(value));
                }
            }
        }

        Ok(EngineExit::BudgetExhausted)
    }
    
    fn execute_one(f: &mut InterpreterFrame) -> StepOutcome {
        let Some(inst) = f.next_inst() else {
            return StepOutcome::new(
                0,
                Err(f.make_exec_error(ExecErrorKind::UnexpectedEOF)),
            );
        };

        let (cost, result) = match inst {
            ILoad(LocalIdx(idx)) => (1, iload(f, idx)),
            IAdd => (2, iadd(f)),
            IReturn => (1, ireturn(f)),

            Unsupported => return StepOutcome::new(
                0,
                Err(f.make_exec_error(ExecErrorKind::UnsupportedInstruction)),
            ),
        };

        StepOutcome::new(cost, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{class_loader::bs_cld::BootstrapCLD, engines::invocation::Invocation, oops::jvalue::JValue};
    #[test]
    fn test_simple_addition() {
        crate::runtime::test_support::init_vm();

        let klass = BootstrapCLD::find_class("SimpleAddition")
            .expect("Failed to load 'SimpleAddition'.");

        let owner = klass.as_normal_klass_ref()
            .expect("Expected NormalKlass.");

        let invocation = Invocation::try_new(
            owner,
            "add",
            "(II)I",
            &[JValue::Int(1), JValue::Int(2)],
        )
        .expect("Failed to create invocation.");

        let mut frame = InterpreterFrame::new(invocation);
        let mut budget = ExecBudget::new(5);

        let exit = Interpreter::execute(
            &mut frame,
            &mut budget,
        )
        .expect("Failed to execute.");
        
        let EngineExit::Return(MethodReturn::Value(JValue::Int(value))) = exit else {
            panic!("Wrong exit.");
        };

        assert!(value == 3, "Wrong result.");
    }
}
