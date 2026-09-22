use crate::{runtime::ms_api::{MsAllocator, MsBox}, code::instructions::Instruction};

#[derive(Debug)]
pub struct Code {
    max_stack: usize,
    max_locals: usize,
    insts: MsBox<[Instruction]>,
}

impl Code {
    pub fn max_locals(&self) -> usize {
        self.max_locals
    }
    pub fn instructions(&self) -> &[Instruction] {
        &self.insts
    }

    pub fn build(cd: &cafebabe::attributes::CodeData, msa: &MsAllocator) -> Self {
        let bytecode = cd.bytecode
            .as_ref()
            .expect("Klover requires bytecode decoding to be enabled.");

        let inst_count = bytecode.opcodes.len();
        let insts = msa.calloc(inst_count);
        for i in 0..inst_count {
            insts[i].write(Instruction::lower(bytecode, i, msa));
        }

        unsafe {
            Self {
                max_locals: cd.max_locals as usize,
                max_stack: cd.max_stack as usize,
                insts: MsBox::from_raw(insts.assume_init_mut())
            }
        }
    }
}
