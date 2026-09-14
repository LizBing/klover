use crate::{class_loader::ms_api::{MSAllocator, MSBox}, code::instructions::Instruction};

pub struct Code {
    pub max_stack: usize,
    pub max_locals: usize,
    pub insts: MSBox<[Instruction]>,
}

impl Code {
    pub fn build(cd: &cafebabe::attributes::CodeData, msa: &MSAllocator) -> Self {
        let opcodes = &cd.bytecode
            .as_ref()
            .expect("Klover requires bytecode decoding to be enabled.")
            .opcodes;

        let insts = msa.calloc(opcodes.len());
        for (i, v) in opcodes.iter().enumerate() {
            insts[i].write(Instruction::from(&v.1));
        }

        unsafe {
            Self {
                max_locals: cd.max_locals as usize,
                max_stack: cd.max_stack as usize,
                insts: MSBox::from_raw(insts.assume_init_mut())
            }
        }
    }
}
