use crate::{class_loader::ms_api::MSRef, engines::{exec_error::{ExecError, ExecResult}, invocation::{Invocation, ResolvedMethod}, slot::Slot}, gc_bindings::oop_hierarchy::NObjPtr, oops::{attr::Code, jvalue::{JByte, JDouble, JFloat, JInt, JLong, JShort, JValue}, method::Method, normal_klass::NormalKlass}};

pub struct InterpreterFrame {
    target: ResolvedMethod,
    
    oprand_stack: Vec<Slot>,
    locals: Box<[Slot]>,

    pc: usize,
    last_pc: usize,
}

impl InterpreterFrame {
    pub fn from_call(target: ResolvedMethod, args: &[Slot]) -> ExecResult<Self> {
        let Some(code) = target.method.code.as_ref() else {
            return Err(ExecError::NoCode);
        };

        let argc = args.len();
        let max_locals = code.max_locals();
        let mut locals = Vec::with_capacity(max_locals);

        for arg in args {
            locals.push(*arg);
        }
        
        for _ in [argc..max_locals] {
            locals.push(Slot::Unused);
        }

        Ok(Self {
            target,
            oprand_stack: Vec::new(),
            locals: locals.into_boxed_slice(),
            pc: 0,
            last_pc: 0,
        })
    }
}

impl InterpreterFrame {
    fn code(&self) -> &Code {
        self.target.method.code.as_ref().unwrap()
    }
}

impl InterpreterFrame {
    pub fn push(&mut self, slot: Slot) {
        self.oprand_stack.push(slot);
    }

    pub fn pop(&mut self) -> ExecResult<Slot> {
        match self.oprand_stack.pop() {
            Some(s) => Ok(s),
            None => Err(ExecError::StackUnderflow),
        }
    }

    pub fn push_long(&mut self, value: JLong) {
        let bytes = value.to_be_bytes();
        
        self.oprand_stack.push(Slot::LongHigh([bytes[0], bytes[1], bytes[2], bytes[3]]));
        self.oprand_stack.push(Slot::LongLow([bytes[4], bytes[5], bytes[6], bytes[7]]));
    }

    pub fn pop_long(&mut self) -> ExecResult<JLong> {
        let Slot::LongLow(low) = self.pop()? else {
            return Err(ExecError::MismatchSlotType);
        };
        
        let Slot::LongHigh(high) = self.pop()? else {
            return Err(ExecError::MismatchSlotType);
        };

        Ok(JLong::from_be_bytes([
            high[0], high[1], high[2], high[3],
            low[0], low[1], low[2], low[3]
        ]))
    }
    pub fn push_double(&mut self, value: JDouble) {
        let bytes = value.to_be_bytes();
        
        self.oprand_stack.push(Slot::DoubleHigh([bytes[0], bytes[1], bytes[2], bytes[3]]));
        self.oprand_stack.push(Slot::DoubleLow([bytes[4], bytes[5], bytes[6], bytes[7]]));
    }

    pub fn pop_double(&mut self) -> ExecResult<JDouble> {
        let Slot::DoubleLow(low) = self.pop()? else {
            return Err(ExecError::MismatchSlotType);
        };
        
        let Slot::DoubleHigh(high) = self.pop()? else {
            return Err(ExecError::MismatchSlotType);
        };

        Ok(JDouble::from_be_bytes([
            high[0], high[1], high[2], high[3],
            low[0], low[1], low[2], low[3]
        ]))
    }

    pub fn get_local(&self, idx: usize) -> ExecResult<Slot> {
        match self.locals.get(idx) {
            Some(s) => Ok(*s),
            None => Err(ExecError::InvalidLocalIndex(idx)),
        }
    }

    pub fn set_local(&mut self, idx: usize, slot: Slot) -> ExecResult<()> {
        match self.locals.get(idx) {
            Some(_) => {
                self.locals[idx] = slot;
                Ok(())
            }
            None => Err(ExecError::InvalidLocalIndex(idx))
        }
    }
}

impl InterpreterFrame {
    pub fn read_u8(&mut self) -> ExecResult<u8> {
        let res;
        match self.code().bytecodes().get(self.pc) {
            Some(b) => res = *b,
            None => return Err(ExecError::EOF),
        }

        self.last_pc = self.pc;
        self.pc += 1;

        Ok(res)
    }

    pub fn read_u16(&mut self) -> ExecResult<u16> {
        let high = self.read_u8()?;
        let low = self.read_u8()?;

        Ok(u16::from_be_bytes([high, low]))
    }

    pub fn read_jbyte(&mut self) -> ExecResult<JByte> {
        let b = self.read_u8()?;
        Ok(b as JByte)
    }

    pub fn read_jshort(&mut self) -> ExecResult<JShort> {
        let high = self.read_u8()?;
        let low = self.read_u8()?;

        Ok(JShort::from_be_bytes([high, low]))
    }
}
