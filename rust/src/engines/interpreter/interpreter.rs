use super::instructions::{
    comparisons::*, constants::*, control::*, conversions::*, loads::*, math::*, stack,
};
use crate::{
    code::instructions::{Instruction, Instruction::*, LocalIdx},
    engines::{
        exec_dispatcher::{EngineExit, ExecBudget},
        exec_error::{ExecErrorKind, ExecResult},
        interpreter::{
            interpreter_frame::InterpreterFrame,
            step_outcome::{StepControl, StepOutcome},
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
                StepControl::Continue => {}
                StepControl::Return(value) => return Ok(EngineExit::Return(value)),
            }
        }
        Ok(EngineExit::BudgetExhausted)
    }

    fn execute_one(f: &mut InterpreterFrame) -> StepOutcome {
        let Some(inst) = f.next_inst() else {
            return StepOutcome::new(0, Err(f.make_exec_error(ExecErrorKind::UnexpectedEOF)));
        };
        // Preserve the existing iadd cost; other supported instructions cost one.
        let cost = match &inst {
            IAdd => 2,
            Unsupported => 0,
            _ => 1,
        };
        StepOutcome::new(cost, Self::execute_instruction(f, inst))
    }

    pub fn execute_instruction(
        f: &mut InterpreterFrame,
        inst: Instruction,
    ) -> ExecResult<StepControl> {
        match inst {
            NOp => nop(f),
            AConstNull => aconst_null(f),
            IConst(value) => iconst(f, value),
            LConst(value) => lconst(f, value),
            FConst(value) => fconst(f, value),
            DConst(value) => dconst(f, value),
            BIPush(value) => bipush(f, value),
            SIPush(value) => sipush(f, value),
            ILoad(LocalIdx(idx)) => iload(f, idx),
            IStore(LocalIdx(idx)) => istore(f, idx),
            LLoad(LocalIdx(idx)) => lload(f, idx),
            LStore(LocalIdx(idx)) => lstore(f, idx),
            FLoad(LocalIdx(idx)) => fload(f, idx),
            FStore(LocalIdx(idx)) => fstore(f, idx),
            DLoad(LocalIdx(idx)) => dload(f, idx),
            DStore(LocalIdx(idx)) => dstore(f, idx),
            ALoad(LocalIdx(idx)) => aload(f, idx),
            AStore(LocalIdx(idx)) => astore(f, idx),
            Pop => stack::pop(f),
            Pop2 => stack::pop2(f),
            Dup => stack::dup(f),
            DupX1 => stack::dup_x1(f),
            DupX2 => stack::dup_x2(f),
            Dup2 => stack::dup2(f),
            Dup2X1 => stack::dup2_x1(f),
            Dup2X2 => stack::dup2_x2(f),
            Swap => stack::swap(f),
            IAdd => iadd(f),
            LAdd => ladd(f),
            FAdd => fadd(f),
            DAdd => dadd(f),
            ISub => isub(f),
            LSub => lsub(f),
            FSub => fsub(f),
            DSub => dsub(f),
            IMul => imul(f),
            LMul => lmul(f),
            FMul => fmul(f),
            DMul => dmul(f),
            IDiv => idiv(f),
            LDiv => ldiv(f),
            FDiv => fdiv(f),
            DDiv => ddiv(f),
            IRem => irem(f),
            LRem => lrem(f),
            FRem => frem(f),
            DRem => drem(f),
            INeg => ineg(f),
            LNeg => lneg(f),
            FNeg => fneg(f),
            DNeg => dneg(f),
            IShL => ishl(f),
            LShL => lshl(f),
            IShR => ishr(f),
            LShR => lshr(f),
            IUShR => iushr(f),
            LUShR => lushr(f),
            IAnd => iand(f),
            LAnd => land(f),
            IOr => ior(f),
            LOr => lor(f),
            IXor => ixor(f),
            LXor => lxor(f),
            IInc(LocalIdx(idx), amount) => iinc(f, idx, amount),
            I2L => i2l(f),
            I2F => i2f(f),
            I2D => i2d(f),
            L2I => l2i(f),
            L2F => l2f(f),
            L2D => l2d(f),
            F2I => f2i(f),
            F2L => f2l(f),
            F2D => f2d(f),
            D2I => d2i(f),
            D2L => d2l(f),
            D2F => d2f(f),
            I2B => i2b(f),
            I2C => i2c(f),
            I2S => i2s(f),
            LCmp => lcmp(f),
            FCmpL => fcmp(f, -1),
            FCmpG => fcmp(f, 1),
            DCmpL => dcmp(f, -1),
            DCmpG => dcmp(f, 1),
            IfEq(target) => ifeq(f, target),
            IfICmpEq(target) => if_icmpeq(f, target),
            IfNe(target) => ifne(f, target),
            IfICmpNe(target) => if_icmpne(f, target),
            IfLt(target) => iflt(f, target),
            IfICmpLt(target) => if_icmplt(f, target),
            IfGe(target) => ifge(f, target),
            IfICmpGe(target) => if_icmpge(f, target),
            IfGt(target) => ifgt(f, target),
            IfICmpGt(target) => if_icmpgt(f, target),
            IfLe(target) => ifle(f, target),
            IfICmpLe(target) => if_icmple(f, target),
            IfACmpEq(target) => if_acmp(f, target, true),
            IfACmpNe(target) => if_acmp(f, target, false),
            IfNull(target) => if_null(f, target, true),
            IfNonNull(target) => if_null(f, target, false),
            Goto(target) => branch(f, target, true),
            TableSwitch(rt) => table_switch(f, &rt),
            LookUpSwitch(lt) => lookup_switch(f, &lt),
            IReturn => ireturn(f),
            LReturn => lreturn(f),
            FReturn => freturn(f),
            DReturn => dreturn(f),
            AReturn => areturn(f),
            Return => return_void(f),
            Unsupported => Err(f.make_exec_error(ExecErrorKind::UnsupportedInstruction)),
        }
    }
}
