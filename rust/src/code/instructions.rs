use crate::{
    class_loader::oop_storage::OOPStorage,
    oops::{
        constant_entries::{LdC2Loadable, LdCLoadable},
        jvalue::{JByte, JDouble, JFloat, JInt, JLong, JShort},
    },
    runtime::ms_api::{MsAllocator, MsBox},
};

#[derive(Debug, Clone, Copy)]
pub struct LocalIdx(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct InstIdx(pub usize);

#[derive(Debug, Clone)]
pub struct RangeTable<'inst> {
    default: InstIdx,
    low: JInt,
    high: JInt,
    jumps: &'inst [InstIdx],
}

impl RangeTable<'_> {
    pub fn get_inst_idx(&self, index: JInt) -> InstIdx {
        if self.low <= index && index <= self.high {
            let idx = (index - self.low) as usize;
            self.jumps[idx]
        } else {
            self.default
        }
    }
}

#[derive(Debug, Clone)]
pub struct LookupTable<'inst> {
    default: InstIdx,
    pairs: &'inst [(JInt, InstIdx)],
}

impl LookupTable<'_> {
    pub fn get_inst_idx(&self, key: JInt) -> InstIdx {
        for iter in self.pairs.iter() {
            let match_value = iter.0;
            if key == match_value {
                return iter.1;
            } else if key < match_value {
                break;
            }
        }

        self.default
    }
}

#[derive(Debug, Clone)]
pub enum Instruction<'inst> {
    // --- Constants ---
    NOp,
    AConstNull,
    IConst(JInt),
    LConst(JLong),
    FConst(JFloat),
    DConst(JDouble),
    BIPush(JByte),
    SIPush(JShort),
    LdC(LdCLoadable),
    LdC2(LdC2Loadable),

    // --- Loads ---
    ILoad(LocalIdx),
    LLoad(LocalIdx),
    FLoad(LocalIdx),
    DLoad(LocalIdx),
    ALoad(LocalIdx),

    // --- Stores ---
    IStore(LocalIdx),
    LStore(LocalIdx),
    FStore(LocalIdx),
    DStore(LocalIdx),
    AStore(LocalIdx),

    // --- Stack ---
    Pop,
    Pop2,
    Dup,
    DupX1,
    DupX2,
    Dup2,
    Dup2X1,
    Dup2X2,
    Swap,

    // --- Math ---
    IAdd,
    LAdd,
    FAdd,
    DAdd,
    ISub,
    LSub,
    FSub,
    DSub,
    IMul,
    LMul,
    FMul,
    DMul,
    IDiv,
    LDiv,
    FDiv,
    DDiv,
    IRem,
    LRem,
    FRem,
    DRem,
    INeg,
    LNeg,
    FNeg,
    DNeg,
    IShL,
    LShL,
    IShR,
    LShR,
    IUShR,
    LUShR,
    IAnd,
    LAnd,
    IOr,
    LOr,
    IXor,
    LXor,
    IInc(LocalIdx, JShort),

    // --- Conversions ---
    I2L,
    I2F,
    I2D,
    L2I,
    L2F,
    L2D,
    F2I,
    F2L,
    F2D,
    D2I,
    D2L,
    D2F,
    I2B,
    I2C,
    I2S,

    // --- Comparison ---
    LCmp,
    FCmpL,
    FCmpG,
    DCmpL,
    DCmpG,
    IfEq(InstIdx),
    IfNe(InstIdx),
    IfLt(InstIdx),
    IfGe(InstIdx),
    IfGt(InstIdx),
    IfLe(InstIdx),
    IfICmpEq(InstIdx),
    IfICmpNe(InstIdx),
    IfICmpLt(InstIdx),
    IfICmpGe(InstIdx),
    IfICmpGt(InstIdx),
    IfICmpLe(InstIdx),
    IfACmpEq(InstIdx),
    IfACmpNe(InstIdx),

    // --- Control ---
    Goto(InstIdx),
    TableSwitch(RangeTable<'inst>),
    LookUpSwitch(LookupTable<'inst>),
    IReturn,
    LReturn,
    FReturn,
    DReturn,
    AReturn,
    Return,

    // --- Extended ---
    IfNull(InstIdx),
    IfNonNull(InstIdx),

    Unsupported,
}

use crate::class_parser::{Bytecode, LoadableConstant, Opcode};

impl<'inst> Instruction<'inst> {
    // Code keeps a one-to-one correspondence with the parser's opcode array.
    pub(super) fn lower(
        bytecodes: &Bytecode,
        opcode_idx: usize,
        msa: &'inst MsAllocator,
        _storage: &OOPStorage,
    ) -> Self {
        let (bci, opcode) = &bytecodes.opcodes[opcode_idx];
        let target = |offset: i32| {
            let bci = usize::try_from(*bci as i64 + i64::from(offset))
                .expect("class parser validated the branch address");
            InstIdx(
                bytecodes
                    .get_opcode_index(bci as u32)
                    .expect("class parser validated the branch instruction boundary"),
            )
        };
        match opcode {
            Opcode::Nop => Self::NOp,
            Opcode::AconstNull => Self::AConstNull,
            Opcode::Iconst(value) => Self::IConst(*value),
            Opcode::Lconst(value) => Self::LConst(*value),
            Opcode::Fconst(value) => Self::FConst(*value),
            Opcode::Dconst(value) => Self::DConst(*value),
            Opcode::Bipush(value) => Self::BIPush(*value),
            Opcode::Sipush(value) => Self::SIPush(*value),
            Opcode::Ldc(loadable) | Opcode::LdcW(loadable) => match loadable.as_ref() {
                LoadableConstant::Integer(value) => Self::LdC(LdCLoadable::Integer(*value)),
                LoadableConstant::Float(value) => Self::LdC(LdCLoadable::Float(*value)),
                _ => Self::Unsupported,
            },
            Opcode::Ldc2W(loadable) => match loadable.as_ref() {
                LoadableConstant::Long(value) => Self::LdC2(LdC2Loadable::Long(*value)),
                LoadableConstant::Double(value) => Self::LdC2(LdC2Loadable::Double(*value)),
                _ => Self::Unsupported,
            },
            Opcode::Iload(index) => Self::ILoad(LocalIdx(*index as usize)),
            Opcode::Istore(index) => Self::IStore(LocalIdx(*index as usize)),
            Opcode::Lload(index) => Self::LLoad(LocalIdx(*index as usize)),
            Opcode::Lstore(index) => Self::LStore(LocalIdx(*index as usize)),
            Opcode::Fload(index) => Self::FLoad(LocalIdx(*index as usize)),
            Opcode::Fstore(index) => Self::FStore(LocalIdx(*index as usize)),
            Opcode::Dload(index) => Self::DLoad(LocalIdx(*index as usize)),
            Opcode::Dstore(index) => Self::DStore(LocalIdx(*index as usize)),
            Opcode::Aload(index) => Self::ALoad(LocalIdx(*index as usize)),
            Opcode::Astore(index) => Self::AStore(LocalIdx(*index as usize)),
            Opcode::Pop => Self::Pop,
            Opcode::Pop2 => Self::Pop2,
            Opcode::Dup => Self::Dup,
            Opcode::DupX1 => Self::DupX1,
            Opcode::DupX2 => Self::DupX2,
            Opcode::Dup2 => Self::Dup2,
            Opcode::Dup2X1 => Self::Dup2X1,
            Opcode::Dup2X2 => Self::Dup2X2,
            Opcode::Swap => Self::Swap,
            Opcode::IAdd => Self::IAdd,
            Opcode::LAdd => Self::LAdd,
            Opcode::FAdd => Self::FAdd,
            Opcode::DAdd => Self::DAdd,
            Opcode::ISub => Self::ISub,
            Opcode::LSub => Self::LSub,
            Opcode::FSub => Self::FSub,
            Opcode::DSub => Self::DSub,
            Opcode::IMul => Self::IMul,
            Opcode::LMul => Self::LMul,
            Opcode::FMul => Self::FMul,
            Opcode::DMul => Self::DMul,
            Opcode::IDiv => Self::IDiv,
            Opcode::LDiv => Self::LDiv,
            Opcode::FDiv => Self::FDiv,
            Opcode::DDiv => Self::DDiv,
            Opcode::IRem => Self::IRem,
            Opcode::LRem => Self::LRem,
            Opcode::FRem => Self::FRem,
            Opcode::DRem => Self::DRem,
            Opcode::INeg => Self::INeg,
            Opcode::LNeg => Self::LNeg,
            Opcode::FNeg => Self::FNeg,
            Opcode::DNeg => Self::DNeg,
            Opcode::IShL => Self::IShL,
            Opcode::LShL => Self::LShL,
            Opcode::IShR => Self::IShR,
            Opcode::LShR => Self::LShR,
            Opcode::IUShR => Self::IUShR,
            Opcode::LUShR => Self::LUShR,
            Opcode::IAnd => Self::IAnd,
            Opcode::LAnd => Self::LAnd,
            Opcode::IOr => Self::IOr,
            Opcode::LOr => Self::LOr,
            Opcode::IXor => Self::IXor,
            Opcode::LXor => Self::LXor,
            Opcode::IInc(index, amount) => Self::IInc(LocalIdx(*index as usize), *amount),
            Opcode::I2L => Self::I2L,
            Opcode::I2F => Self::I2F,
            Opcode::I2D => Self::I2D,
            Opcode::L2I => Self::L2I,
            Opcode::L2F => Self::L2F,
            Opcode::L2D => Self::L2D,
            Opcode::F2I => Self::F2I,
            Opcode::F2L => Self::F2L,
            Opcode::F2D => Self::F2D,
            Opcode::D2I => Self::D2I,
            Opcode::D2L => Self::D2L,
            Opcode::D2F => Self::D2F,
            Opcode::I2B => Self::I2B,
            Opcode::I2C => Self::I2C,
            Opcode::I2S => Self::I2S,
            Opcode::LCmp => Self::LCmp,
            Opcode::FCmpL => Self::FCmpL,
            Opcode::FCmpG => Self::FCmpG,
            Opcode::DCmpL => Self::DCmpL,
            Opcode::DCmpG => Self::DCmpG,
            Opcode::IfEq(offset) => Self::IfEq(target(*offset)),
            Opcode::IfICmpEq(offset) => Self::IfICmpEq(target(*offset)),
            Opcode::IfNe(offset) => Self::IfNe(target(*offset)),
            Opcode::IfICmpNe(offset) => Self::IfICmpNe(target(*offset)),
            Opcode::IfLt(offset) => Self::IfLt(target(*offset)),
            Opcode::IfICmpLt(offset) => Self::IfICmpLt(target(*offset)),
            Opcode::IfGe(offset) => Self::IfGe(target(*offset)),
            Opcode::IfICmpGe(offset) => Self::IfICmpGe(target(*offset)),
            Opcode::IfGt(offset) => Self::IfGt(target(*offset)),
            Opcode::IfICmpGt(offset) => Self::IfICmpGt(target(*offset)),
            Opcode::IfLe(offset) => Self::IfLe(target(*offset)),
            Opcode::IfICmpLe(offset) => Self::IfICmpLe(target(*offset)),
            Opcode::IfACmpEq(offset) => Self::IfACmpEq(target(*offset)),
            Opcode::IfACmpNe(offset) => Self::IfACmpNe(target(*offset)),
            Opcode::IfNull(offset) => Self::IfNull(target(*offset)),
            Opcode::IfNonNull(offset) => Self::IfNonNull(target(*offset)),
            Opcode::Goto(offset) => Self::Goto(target(*offset)),
            Opcode::TableSwitch(table) => {
                let mut jumps = MsBox::new_uninit_slice(msa, table.jumps.len());
                for (slot, offset) in jumps.iter_mut().zip(&table.jumps) {
                    slot.write(target(*offset));
                }
                Self::TableSwitch(RangeTable {
                    default: target(table.default),
                    low: table.low,
                    high: table.high,
                    // Every slot is initialized; targets contain no resources.
                    jumps: unsafe { jumps.assume_init() }.leak(),
                })
            }
            Opcode::LookupSwitch(table) => {
                let mut pairs = MsBox::new_uninit_slice(msa, table.match_offsets.len());
                for (slot, (key, offset)) in pairs.iter_mut().zip(&table.match_offsets) {
                    slot.write((*key, target(*offset)));
                }
                Self::LookUpSwitch(LookupTable {
                    default: target(table.default),
                    pairs: unsafe { pairs.assume_init() }.leak(),
                })
            }
            Opcode::IReturn => Self::IReturn,
            Opcode::LReturn => Self::LReturn,
            Opcode::FReturn => Self::FReturn,
            Opcode::DReturn => Self::DReturn,
            Opcode::AReturn => Self::AReturn,
            Opcode::Return => Self::Return,
            Opcode::GetStatic(_)
            | Opcode::PutStatic(_)
            | Opcode::GetField(_)
            | Opcode::PutField(_)
            | Opcode::InvokeVirtual(_)
            | Opcode::InvokeSpecial(_)
            | Opcode::InvokeStatic(_)
            | Opcode::InvokeInterface { .. }
            | Opcode::InvokeDynamic(_)
            | Opcode::New(_)
            | Opcode::Anewarray(_)
            | Opcode::Checkcast(_)
            | Opcode::Instanceof(_)
            | Opcode::Multianewarray { .. }
            | Opcode::Unsupported(_) => Self::Unsupported,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::class_loader::bs_cld::BootstrapCLD;

    use super::*;

    #[test]
    fn lower_branches_use_byte_addresses_not_instruction_indices() {
        let msa = BootstrapCLD::ms_allocator();
        let storage = OOPStorage::new(msa);

        let code = Bytecode {
            opcodes: vec![
                (0, Opcode::Bipush(10)),
                (2, Opcode::Istore(0)),
                (3, Opcode::Goto(5)),
                (6, Opcode::Bipush(20)),
                (8, Opcode::Iload(0)),
                (9, Opcode::IfNe(-9)),
                (12, Opcode::IReturn),
            ],
        };
        assert!(matches!(
            Instruction::lower(&code, 2, msa, &storage),
            Instruction::Goto(InstIdx(4))
        ));
        assert!(matches!(
            Instruction::lower(&code, 5, msa, &storage),
            Instruction::IfNe(InstIdx(0))
        ));
        // goto_w is normalized by the parser to the same Opcode::Goto variant.
        let wide = Bytecode {
            opcodes: vec![(0, Opcode::Goto(40_000)), (40_000, Opcode::Return)],
        };
        assert!(matches!(
            Instruction::lower(&wide, 0, msa, &storage),
            Instruction::Goto(InstIdx(1))
        ));
    }

    #[test]
    fn lower_retains_wide_local_indices_and_signed_increments() {
        let msa = BootstrapCLD::ms_allocator();
        let storage = OOPStorage::new(msa);

        let code = Bytecode {
            opcodes: vec![
                (0, Opcode::IInc(300, -1000)),
                (6, Opcode::Iload(300)),
                (10, Opcode::Lstore(400)),
            ],
        };
        assert!(matches!(
            Instruction::lower(&code, 0, msa, &storage),
            Instruction::IInc(LocalIdx(300), -1000)
        ));
        assert!(matches!(
            Instruction::lower(&code, 1, msa, &storage),
            Instruction::ILoad(LocalIdx(300))
        ));
        assert!(matches!(
            Instruction::lower(&code, 2, msa, &storage),
            Instruction::LStore(LocalIdx(400))
        ));
    }

    #[test]
    fn lower_stack_variants_without_losing_their_forms() {
        let msa = BootstrapCLD::ms_allocator();
        let storage = OOPStorage::new(msa);

        let opcodes = [
            Opcode::Nop,
            Opcode::Pop,
            Opcode::Pop2,
            Opcode::Dup,
            Opcode::DupX1,
            Opcode::DupX2,
            Opcode::Dup2,
            Opcode::Dup2X1,
            Opcode::Dup2X2,
            Opcode::Swap,
        ];
        let expected = [
            Instruction::NOp,
            Instruction::Pop,
            Instruction::Pop2,
            Instruction::Dup,
            Instruction::DupX1,
            Instruction::DupX2,
            Instruction::Dup2,
            Instruction::Dup2X1,
            Instruction::Dup2X2,
            Instruction::Swap,
        ];
        let code = Bytecode {
            opcodes: opcodes
                .into_iter()
                .enumerate()
                .map(|(i, op)| (i as u32, op))
                .collect(),
        };
        for (idx, expected) in expected.iter().enumerate() {
            assert_eq!(
                std::mem::discriminant(&Instruction::lower(&code, idx, msa, &storage)),
                std::mem::discriminant(expected)
            );
        }
    }
}
