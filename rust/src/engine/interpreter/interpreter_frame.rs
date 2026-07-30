use crate::{
    engine::{
        engine_error::{ExecError, ExecResult},
        interpreter::operand_stack::{OperandStack, StackValue},
        outcome::RetValue,
        resolved_method::ResolvedMethod,
        slot::Slot,
    },
    oops::{attr::Code, cp_entry::CPEntry},
};


#[derive(Debug)]
pub struct InterpreterFrame {
    target: ResolvedMethod,

    pc: usize,
    last_pc: usize,
    
    locals: Box<[Slot]>,
    opstack: OperandStack,

    reserved_slots: usize,
}

impl InterpreterFrame {
    pub fn new(
        target: ResolvedMethod,
        args: &[Slot],
    ) -> ExecResult<Self> {
        let method = target.method();
        let code = method
            .code
            .as_ref()
            .ok_or(ExecError::MethodHasNoCode)?;

        let max_stack = code.max_stack;
        let max_locals = code.max_locals;

        if args.len() > max_locals {
            return Err(ExecError::TooManyArguments {
                args: args.len(),
                max_locals,
            });
        }

        let mut locals =
            vec![Slot::EMPTY; max_locals].into_boxed_slice();

        locals[..args.len()].copy_from_slice(args);

        Ok(Self {
            target,
            pc: 0,
            last_pc: 0,
            locals,
            opstack: OperandStack::new(max_stack),
            reserved_slots: max_locals + max_stack
        })
    }
}

impl InterpreterFrame {
    pub fn code(&self) -> &Code {
        self.target.method().code.as_ref().unwrap()
    }

    pub fn reserved_slots(&self) -> usize {
        self.reserved_slots
    }

    pub fn constant_pool_entry(&self, index: usize) -> Option<&CPEntry> {
        self.target.holder().constant_pool_entry(index)
    }
}

impl InterpreterFrame {
    pub fn push(&mut self, slot: Slot) -> ExecResult<()> {
        self.opstack.push_slot(slot)
    }
    
    pub fn pop(&mut self) -> ExecResult<Slot> {
        self.opstack.pop_slot()
    }
    
    pub fn get_local(&self, index: usize) -> ExecResult<Slot> {
        self.locals
            .get(index)
            .copied()
            .ok_or(ExecError::InvalidLocalIndex(index))
    }

    pub fn set_local(&mut self, index: usize, value: Slot) -> ExecResult<()> {
        let local = self
            .locals
            .get_mut(index)
            .ok_or(ExecError::InvalidLocalIndex(index))?;
        *local = value;
        Ok(())
    }

    pub fn push_long(&mut self, value: i64) -> ExecResult<()> {
        self.opstack.push_value(StackValue::Category2(
            Slot::long_high(value),
            Slot::long_low(value),
        ))
    }

    pub fn pop_long(&mut self) -> ExecResult<i64> {
        let StackValue::Category2(high, low) = self.opstack.top_values(1)?[0] else {
            return Err(ExecError::InvalidOperandStackShape);
        };
        let value = Slot::as_long(high, low)?;
        self.opstack.replace_top_values(1, &[])?;
        Ok(value)
    }

    pub fn push_double(&mut self, value: f64) -> ExecResult<()> {
        self.opstack.push_value(StackValue::Category2(
            Slot::double_high(value),
            Slot::double_low(value),
        ))
    }

    pub fn pop_double(&mut self) -> ExecResult<f64> {
        let StackValue::Category2(high, low) = self.opstack.top_values(1)?[0] else {
            return Err(ExecError::InvalidOperandStackShape);
        };
        let value = Slot::as_double(high, low)?;
        self.opstack.replace_top_values(1, &[])?;
        Ok(value)
    }

    pub(crate) fn operand_stack_mut(&mut self) -> &mut OperandStack {
        &mut self.opstack
    }
    
    pub fn push_return_value(
        &mut self,
        value: RetValue,
    ) -> ExecResult<()> {
        match value {
            RetValue::Void => Ok(()),
    
            RetValue::Int(value) => {
                self.push(Slot::int(value))
            }
    
            RetValue::Float(value) => {
                self.push(Slot::float(value))
            }
    
            RetValue::Ref(value) => {
                self.push(Slot::reference(value))
            }
    
            RetValue::Long(value) => {
                self.push_long(value)
            }
    
            RetValue::Double(value) => {
                self.push_double(value)
            }
        }
    }
}

impl InterpreterFrame {
    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn last_pc(&self) -> usize {
        self.last_pc
    }

    pub fn set_pc(&mut self, target: usize) -> ExecResult<()> {
        let code_len = self.code().bytecodes.len();
    
        if target >= code_len {
            return Err(ExecError::InvalidProgramCounter {
                target,
                code_len,
            });
        }
    
        self.pc = target;
        Ok(())
    }
    
    pub fn fetch_opcode(&mut self) -> ExecResult<u8> {
        self.last_pc = self.pc;
        self.read_u8()
    }

    pub fn read_u8(&mut self) -> ExecResult<u8> {
        let value = self
            .code()
            .bytecodes
            .get(self.pc)
            .copied()
            .ok_or(ExecError::UnexpectedEndOfCode {
                bci: self.pc,
            })?;

        self.pc += 1;
        Ok(value)
    }

    pub fn read_i8(&mut self) -> ExecResult<i8> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_u16(&mut self) -> ExecResult<u16> {
        Ok(u16::from_be_bytes([
            self.read_u8()?,
            self.read_u8()?,
        ]))
    }

    pub fn read_i16(&mut self) -> ExecResult<i16> {
        Ok(self.read_u16()? as i16)
    }

    pub fn branch_target(&self, offset: i16) -> ExecResult<usize> {
        let target = self
            .last_pc
            .checked_add_signed(offset as isize)
            .ok_or(ExecError::InvalidBranchTarget {
                from: self.last_pc,
                offset: offset as i32,
            })?;

        if target >= self.code().bytecodes.len() {
            return Err(ExecError::InvalidBranchTarget {
                from: self.last_pc,
                offset: offset as i32,
            });
        }

        Ok(target)
    }
}

impl InterpreterFrame {
    pub fn take_top_slots(&mut self, arg_slots: usize) -> ExecResult<Vec<Slot>> {
        self.opstack.take_top_slots(arg_slots)
    }
}
