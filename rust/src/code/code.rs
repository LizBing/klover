use crate::{class_loader::oop_storage::OOPStorage, code::instructions::Instruction, runtime::ms_api::{MsAllocator, MsBox}};

#[derive(Debug)]
pub struct Code<'method> {
    max_stack: usize,
    max_locals: usize,
    insts: &'method [Instruction<'method>],
}

impl<'method> Code<'method> {
    pub fn max_locals(&self) -> usize {
        self.max_locals
    }
    pub fn instructions(&self) -> &[Instruction<'method>] {
        &self.insts
    }

    pub fn build(cd: &cafebabe::attributes::CodeData, msa: &'method MsAllocator, storage: &OOPStorage) -> Self {
        let bytecode = cd.bytecode
            .as_ref()
            .expect("Klover requires bytecode decoding to be enabled.");

        let inst_count = bytecode.opcodes.len();
        let mut insts = MsBox::new_uninit_slice(msa, inst_count);
        for i in 0..inst_count {
            insts[i].write(Instruction::lower(bytecode, i, msa, storage));
        }

        Self {
            max_locals: cd.max_locals as usize,
            max_stack: cd.max_stack as usize,
            insts: unsafe { insts.assume_init().leak() },
        }
    }
}
