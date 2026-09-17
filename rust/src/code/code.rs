use crate::{runtime::ms_api::{MsAllocator, MsBox}, code::instructions::Instruction};

#[derive(Debug)]
pub struct Code {
    pub max_stack: usize,
    pub max_locals: usize,
    pub insts: MsBox<[Instruction]>,
}

impl Code {
    pub fn build(cd: &cafebabe::attributes::CodeData, msa: &MsAllocator) -> Self {
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
                insts: MsBox::from_raw(insts.assume_init_mut())
            }
        }
    }
}
