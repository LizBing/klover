use crate::{
    class_loader::oop_storage::OopStorage, class_parser::CodeData, code::instructions::Instruction, oops::tmp_constant_pool::TmpConstantPool, runtime::ms_api::{MsAllocator, MsBox}
};

#[derive(Debug)]
pub struct Code<'method> {
    _private: (),

    pub max_stack: usize,
    pub max_locals: usize,
    pub insts: &'method [Instruction<'method>],
}

impl<'method> Code<'method> {
    pub fn build(cd: &CodeData, msa: &'method MsAllocator, storage: &OopStorage) -> Self {
        let bytecode = &cd.bytecode;
        let mut tmp_cp = TmpConstantPool::new();

        let inst_count = bytecode.opcodes.len();
        let mut insts = MsBox::new_uninit_slice(msa, inst_count);
        for i in 0..inst_count {
            insts[i].write(Instruction::lower(bytecode, i, msa, &mut tmp_cp, storage));
        }

        Self {
            _private: (),
            max_locals: cd.max_locals as usize,
            max_stack: cd.max_stack as usize,
            insts: unsafe { insts.assume_init().leak() },
        }
    }
}
