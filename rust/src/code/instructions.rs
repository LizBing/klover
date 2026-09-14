#[repr(transparent)]
#[derive(Clone)]
pub struct LocalIdx(pub usize);

#[derive(Clone)]
pub enum Instruction {
    ILoad(LocalIdx),
    IAdd,
    IReturn,

    Unsupported,
}

use cafebabe::bytecode::Opcode::*;
impl From<&cafebabe::bytecode::Opcode<'_>> for Instruction {
    fn from(value: &cafebabe::bytecode::Opcode<'_>) -> Self {
        match value {
            Iload(idx) => Self::ILoad(LocalIdx(*idx as usize)),
            Iadd => Self::IAdd,
            Ireturn => Self::IReturn,

            _ => Self::Unsupported,
        }
    }
}
