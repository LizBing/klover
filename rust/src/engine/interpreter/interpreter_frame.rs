use crate::{engine::{engine_error::{ExecError, ExecResult}, outcome::RetValue, resolved_method::ResolvedMethod, slot::Slot}, oops::{attr::Code, method::Method, normal_klass::NormalKlass}};


#[derive(Debug)]
pub struct InterpreterFrame {
    target: ResolvedMethod,

    pc: usize,
    last_pc: usize,
    
    locals: Box<[Slot]>,
    opstack: Vec<Slot>,

    max_stack: usize,
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
            opstack: Vec::with_capacity(max_stack),
            max_stack,
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
}

impl InterpreterFrame {
    pub fn push(&mut self, slot: Slot) -> ExecResult<()> {
        if self.opstack.len() >= self.max_stack {
            return Err(ExecError::OperandStackOverflow);
        }
    
        self.opstack.push(slot);
        Ok(())
    }
    
    pub fn pop(&mut self) -> ExecResult<Slot> {
        self.opstack
            .pop()
            .ok_or(ExecError::OperandStackUnderflow)
    }
    
    pub fn get_local(&self, index: usize) -> ExecResult<Slot> {
        self.locals
            .get(index)
            .copied()
            .ok_or(ExecError::InvalidLocalIndex(index))
    }

    fn ensure_stack_capacity(
        &self,
        additional: usize,
    ) -> ExecResult<()> {
        let new_len = self
            .opstack
            .len()
            .checked_add(additional)
            .ok_or(ExecError::OperandStackOverflow)?;
    
        if new_len > self.max_stack {
            return Err(ExecError::OperandStackOverflow);
        }
    
        Ok(())
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
                self.ensure_stack_capacity(2)?;
    
                self.opstack.push(Slot::long_high(value));
                self.opstack.push(Slot::long_low(value));
    
                Ok(())
            }
    
            RetValue::Double(value) => {
                self.ensure_stack_capacity(2)?;
    
                self.opstack.push(Slot::double_high(value));
                self.opstack.push(Slot::double_low(value));
    
                Ok(())
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
        if arg_slots > self.opstack.len() {
            return Err(ExecError::OperandStackUnderflow);
        }
        
        let start = self.opstack.len() - arg_slots;
        Ok(self.opstack.split_off(start))
    }
}
