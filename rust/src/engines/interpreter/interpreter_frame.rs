use crate::{
    code::instructions::{InstIdx, Instruction},
    engines::{
        exec_dispatcher::{EngineExit, ExecBudget, MethodReturn},
        exec_error::{ExecError, ExecErrorKind, ExecResult},
        interpreter::{interpreter::Interpreter, slot::Slot},
        invocation::Invocation,
        java_frame::JavaFrame,
    },
    oops::jvalue::{JDouble, JInt, JLong, JValue},
};

pub struct InterpreterFrame {
    invocation: Invocation,

    oprand_stack: Vec<Slot>,
    locals: Box<[Slot]>,

    pc: usize,
    last_pc: usize,
}

impl InterpreterFrame {
    // Todo: make try_new()
    pub fn new(invocation: Invocation) -> Self {
        let oprand_stack = Vec::new();

        let mut locals: Box<[Slot]> = std::iter::repeat(Slot::Unused)
            .take(invocation.code().max_locals())
            .collect();

        let mut idx = 0;
        for arg in invocation.args().iter() {
            match *arg {
                JValue::Boolean(b) => locals[idx] = Slot::Int(b as JInt),
                JValue::Byte(v) => locals[idx] = Slot::Int(v.into()),
                JValue::Short(v) => locals[idx] = Slot::Int(v.into()),
                JValue::Char(v) => locals[idx] = Slot::Int(v.into()),
                JValue::Int(v) => locals[idx] = Slot::Int(v),
                JValue::Float(v) => locals[idx] = Slot::Float(v),
                JValue::Ref(v) => locals[idx] = Slot::Ref(v),
                JValue::Long(v) => {
                    let bytes = v.to_be_bytes();
                    locals[idx] = Slot::LongHigh([bytes[0], bytes[1], bytes[2], bytes[3]]);
                    locals[idx + 1] = Slot::LongLow([bytes[4], bytes[5], bytes[6], bytes[7]]);
                    idx += 1;
                }
                JValue::Double(v) => {
                    let bytes = v.to_be_bytes();
                    locals[idx] = Slot::DoubleHigh([bytes[0], bytes[1], bytes[2], bytes[3]]);
                    locals[idx + 1] = Slot::DoubleLow([bytes[4], bytes[5], bytes[6], bytes[7]]);
                    idx += 1;
                }
            }
            idx += 1;
        }

        Self {
            invocation,
            oprand_stack,
            locals,
            pc: 0,
            last_pc: 0,
        }
    }
}

impl InterpreterFrame {
    pub(super) fn make_exec_error(&self, kind: ExecErrorKind) -> ExecError {
        ExecError::with_invocation(self.invocation.clone(), kind)
    }
}

impl JavaFrame for InterpreterFrame {
    fn resume(&mut self, budget: &mut ExecBudget) -> ExecResult<EngineExit> {
        Interpreter::execute(self, budget)
    }

    fn accept_return(&mut self, value: MethodReturn) -> ExecResult<()> {
        let jvalue = match value {
            MethodReturn::Void => return Ok(()),
            MethodReturn::Value(v) => v,
        };

        match jvalue {
            JValue::Boolean(b) => self.push(Slot::Int(b as JInt)),
            JValue::Byte(b) => self.push(Slot::Int(b as JInt)),
            JValue::Char(c) => self.push(Slot::Int(c as JInt)),
            JValue::Double(d) => self.push_double(d),
            JValue::Float(f) => self.push(Slot::Float(f)),
            JValue::Int(i) => self.push(Slot::Int(i)),
            JValue::Long(l) => self.push_long(l),
            JValue::Ref(r) => self.push(Slot::Ref(r)),
            JValue::Short(s) => self.push(Slot::Int(s as JInt)),
        }

        Ok(())
    }
}

impl InterpreterFrame {
    pub(super) fn next_inst(&mut self) -> Option<Instruction> {
        let res = self.invocation.code().instructions().get(self.pc).cloned();
        if res.is_some() {
            self.last_pc = self.pc;
            self.pc += 1;
        }

        res
    }
}

impl InterpreterFrame {
    pub(super) fn push(&mut self, slot: Slot) {
        self.oprand_stack.push(slot);
    }

    pub(super) fn pop(&mut self) -> ExecResult<Slot> {
        match self.oprand_stack.pop() {
            Some(s) => Ok(s),
            None => Err(self.make_exec_error(ExecErrorKind::StackUnderflow)),
        }
    }

    pub(super) fn push_long(&mut self, value: JLong) {
        let bytes = value.to_be_bytes();

        self.oprand_stack
            .push(Slot::LongHigh([bytes[0], bytes[1], bytes[2], bytes[3]]));
        self.oprand_stack
            .push(Slot::LongLow([bytes[4], bytes[5], bytes[6], bytes[7]]));
    }

    pub(super) fn pop_long(&mut self) -> ExecResult<JLong> {
        let Slot::LongLow(low) = self.pop()? else {
            return Err(self.make_exec_error(ExecErrorKind::MismatchSlotType));
        };

        let Slot::LongHigh(high) = self.pop()? else {
            return Err(self.make_exec_error(ExecErrorKind::MismatchSlotType));
        };

        Ok(JLong::from_be_bytes([
            high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3],
        ]))
    }

    pub(super) fn push_double(&mut self, value: JDouble) {
        let bytes = value.to_be_bytes();

        self.oprand_stack
            .push(Slot::DoubleHigh([bytes[0], bytes[1], bytes[2], bytes[3]]));
        self.oprand_stack
            .push(Slot::DoubleLow([bytes[4], bytes[5], bytes[6], bytes[7]]));
    }

    pub(super) fn pop_double(&mut self) -> ExecResult<JDouble> {
        let Slot::DoubleLow(low) = self.pop()? else {
            return Err(self.make_exec_error(ExecErrorKind::MismatchSlotType));
        };

        let Slot::DoubleHigh(high) = self.pop()? else {
            return Err(self.make_exec_error(ExecErrorKind::MismatchSlotType));
        };

        Ok(JDouble::from_be_bytes([
            high[0], high[1], high[2], high[3], low[0], low[1], low[2], low[3],
        ]))
    }

    pub(super) fn get_local(&self, idx: usize) -> ExecResult<Slot> {
        match self.locals.get(idx) {
            Some(s) => Ok(*s),
            None => Err(self.make_exec_error(ExecErrorKind::InvalidLocalIndex(idx))),
        }
    }

    pub(super) fn set_local(&mut self, idx: usize, slot: Slot) -> ExecResult<()> {
        match self.locals.get(idx) {
            Some(_) => {
                self.invalidate_local(idx);
                self.locals[idx] = slot;
                Ok(())
            }
            None => Err(self.make_exec_error(ExecErrorKind::InvalidLocalIndex(idx))),
        }
    }
}

impl InterpreterFrame {
    pub(super) fn jump_to(&mut self, target: InstIdx) -> ExecResult<()> {
        if target.0 >= self.invocation.code().instructions().len() {
            return Err(self.make_exec_error(ExecErrorKind::InvalidBranchTarget(target.0)));
        }
        self.pc = target.0;
        Ok(())
    }

    pub(super) fn operand_stack(&self) -> &[Slot] {
        &self.oprand_stack
    }

    // Used only after stack instructions have checked the entire replaced suffix.
    pub(super) fn replace_stack_top(&mut self, count: usize, replacement: &[Slot]) {
        self.oprand_stack.truncate(self.oprand_stack.len() - count);
        self.oprand_stack.extend_from_slice(replacement);
    }

    fn invalidate_local(&mut self, idx: usize) {
        match self.locals[idx] {
            Slot::LongHigh(_) | Slot::DoubleHigh(_) if idx + 1 < self.locals.len() => {
                self.locals[idx + 1] = Slot::Unused;
            }
            Slot::LongLow(_) | Slot::DoubleLow(_) if idx > 0 => {
                self.locals[idx - 1] = Slot::Unused;
            }
            _ => {}
        }
        self.locals[idx] = Slot::Unused;
    }

    pub(super) fn set_wide_local(&mut self, idx: usize, high: Slot, low: Slot) -> ExecResult<()> {
        self.get_local(idx)?;
        self.get_local(idx + 1)?;
        // Invalidate both old values before writing either new slot.
        self.invalidate_local(idx);
        self.invalidate_local(idx + 1);
        self.locals[idx] = high;
        self.locals[idx + 1] = low;
        Ok(())
    }
}

impl InterpreterFrame {
    pub(super) fn push_int(&mut self, value: JInt) {
        self.push(Slot::Int(value));
    }

    pub(super) fn pop_int(&mut self) -> ExecResult<JInt> {
        match self.pop()? {
            Slot::Int(value) => Ok(value),
            _ => Err(self.make_exec_error(ExecErrorKind::MismatchSlotType)),
        }
    }
}

impl InterpreterFrame {
    pub(super) fn push_float(&mut self, value: crate::oops::jvalue::JFloat) {
        self.push(Slot::Float(value));
    }

    pub(super) fn pop_float(&mut self) -> ExecResult<crate::oops::jvalue::JFloat> {
        match self.pop()? {
            Slot::Float(value) => Ok(value),
            _ => Err(self.make_exec_error(ExecErrorKind::MismatchSlotType)),
        }
    }
}

impl InterpreterFrame {
    pub(super) fn push_ref(&mut self, value: crate::gc_bindings::oop_hierarchy::NObjPtr) {
        self.push(Slot::Ref(value));
    }

    pub(super) fn pop_ref(&mut self) -> ExecResult<crate::gc_bindings::oop_hierarchy::NObjPtr> {
        match self.pop()? {
            Slot::Ref(value) => Ok(value),
            _ => Err(self.make_exec_error(ExecErrorKind::MismatchSlotType)),
        }
    }
}
