use crate::oops::jvalue::{JByte, JDouble, JFloat, JInt, JLong, JShort};

#[derive(Debug, Clone)]
pub struct LocalIdx(pub usize);

#[derive(Debug, Clone)]
pub struct InstIdx(pub usize);

#[derive(Debug, Clone)]
pub enum Instruction {
    // --- Constants ---
    NOp,
    AConstNull,
    IConst(JInt),
    LConst(JLong),
    FConst(JFloat),
    DConst(JDouble),
    BIPush(JByte),
    SIPush(JShort),

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

use cafebabe::bytecode::{ByteCode, Opcode};

impl Instruction {
    // Code keeps a one-to-one correspondence with the parser's opcode array.
    pub(super) fn lower(bytecodes: &ByteCode<'_>, opcode_idx: usize) -> Self {
        let (bci, opcode) = &bytecodes.opcodes[opcode_idx];
        let target = |offset: i32| {
            let bci = usize::try_from(*bci as i64 + i64::from(offset))
                .expect("cafebabe validated the branch address");
            InstIdx(
                bytecodes
                    .get_opcode_index(bci)
                    .expect("cafebabe validated the branch instruction boundary"),
            )
        };
        match opcode {
            Opcode::Nop => Self::NOp,
            Opcode::AconstNull => Self::AConstNull,
            Opcode::Iconst0 => Self::IConst(0),
            Opcode::Iconst1 => Self::IConst(1),
            Opcode::Iconst2 => Self::IConst(2),
            Opcode::Iconst3 => Self::IConst(3),
            Opcode::Iconst4 => Self::IConst(4),
            Opcode::Iconst5 => Self::IConst(5),
            Opcode::Lconst0 => Self::LConst(0),
            Opcode::Lconst1 => Self::LConst(1),
            Opcode::Fconst0 => Self::FConst(0.0),
            Opcode::Fconst1 => Self::FConst(1.0),
            Opcode::Fconst2 => Self::FConst(2.0),
            Opcode::Dconst0 => Self::DConst(0.0),
            Opcode::Dconst1 => Self::DConst(1.0),
            Opcode::IconstM1 => Self::IConst(-1),
            Opcode::Bipush(v) => Self::BIPush(*v),
            Opcode::Sipush(v) => Self::SIPush(*v),
            Opcode::Iload(idx) => Self::ILoad(LocalIdx(*idx as usize)),
            Opcode::Istore(idx) => Self::IStore(LocalIdx(*idx as usize)),
            Opcode::Lload(idx) => Self::LLoad(LocalIdx(*idx as usize)),
            Opcode::Lstore(idx) => Self::LStore(LocalIdx(*idx as usize)),
            Opcode::Fload(idx) => Self::FLoad(LocalIdx(*idx as usize)),
            Opcode::Fstore(idx) => Self::FStore(LocalIdx(*idx as usize)),
            Opcode::Dload(idx) => Self::DLoad(LocalIdx(*idx as usize)),
            Opcode::Dstore(idx) => Self::DStore(LocalIdx(*idx as usize)),
            Opcode::Aload(idx) => Self::ALoad(LocalIdx(*idx as usize)),
            Opcode::Astore(idx) => Self::AStore(LocalIdx(*idx as usize)),
            Opcode::Pop => Self::Pop,
            Opcode::Pop2 => Self::Pop2,
            Opcode::Dup => Self::Dup,
            Opcode::DupX1 => Self::DupX1,
            Opcode::DupX2 => Self::DupX2,
            Opcode::Dup2 => Self::Dup2,
            Opcode::Dup2X1 => Self::Dup2X1,
            Opcode::Dup2X2 => Self::Dup2X2,
            Opcode::Swap => Self::Swap,
            Opcode::Iadd => Self::IAdd,
            Opcode::Ladd => Self::LAdd,
            Opcode::Fadd => Self::FAdd,
            Opcode::Dadd => Self::DAdd,
            Opcode::Isub => Self::ISub,
            Opcode::Lsub => Self::LSub,
            Opcode::Fsub => Self::FSub,
            Opcode::Dsub => Self::DSub,
            Opcode::Imul => Self::IMul,
            Opcode::Lmul => Self::LMul,
            Opcode::Fmul => Self::FMul,
            Opcode::Dmul => Self::DMul,
            Opcode::Idiv => Self::IDiv,
            Opcode::Ldiv => Self::LDiv,
            Opcode::Fdiv => Self::FDiv,
            Opcode::Ddiv => Self::DDiv,
            Opcode::Irem => Self::IRem,
            Opcode::Lrem => Self::LRem,
            Opcode::Frem => Self::FRem,
            Opcode::Drem => Self::DRem,
            Opcode::Ineg => Self::INeg,
            Opcode::Lneg => Self::LNeg,
            Opcode::Fneg => Self::FNeg,
            Opcode::Dneg => Self::DNeg,
            Opcode::Ishl => Self::IShL,
            Opcode::Lshl => Self::LShL,
            Opcode::Ishr => Self::IShR,
            Opcode::Lshr => Self::LShR,
            Opcode::Iushr => Self::IUShR,
            Opcode::Lushr => Self::LUShR,
            Opcode::Iand => Self::IAnd,
            Opcode::Land => Self::LAnd,
            Opcode::Ior => Self::IOr,
            Opcode::Lor => Self::LOr,
            Opcode::Ixor => Self::IXor,
            Opcode::Lxor => Self::LXor,
            Opcode::Iinc(idx, amount) => Self::IInc(LocalIdx(*idx as usize), *amount),
            Opcode::I2l => Self::I2L,
            Opcode::I2f => Self::I2F,
            Opcode::I2d => Self::I2D,
            Opcode::L2i => Self::L2I,
            Opcode::L2f => Self::L2F,
            Opcode::L2d => Self::L2D,
            Opcode::F2i => Self::F2I,
            Opcode::F2l => Self::F2L,
            Opcode::F2d => Self::F2D,
            Opcode::D2i => Self::D2I,
            Opcode::D2l => Self::D2L,
            Opcode::D2f => Self::D2F,
            Opcode::I2b => Self::I2B,
            Opcode::I2c => Self::I2C,
            Opcode::I2s => Self::I2S,
            Opcode::Lcmp => Self::LCmp,
            Opcode::Fcmpl => Self::FCmpL,
            Opcode::Fcmpg => Self::FCmpG,
            Opcode::Dcmpl => Self::DCmpL,
            Opcode::Dcmpg => Self::DCmpG,
            Opcode::Ifeq(offset) => Self::IfEq(target(*offset)),
            Opcode::IfIcmpeq(offset) => Self::IfICmpEq(target(*offset)),
            Opcode::Ifne(offset) => Self::IfNe(target(*offset)),
            Opcode::IfIcmpne(offset) => Self::IfICmpNe(target(*offset)),
            Opcode::Iflt(offset) => Self::IfLt(target(*offset)),
            Opcode::IfIcmplt(offset) => Self::IfICmpLt(target(*offset)),
            Opcode::Ifge(offset) => Self::IfGe(target(*offset)),
            Opcode::IfIcmpge(offset) => Self::IfICmpGe(target(*offset)),
            Opcode::Ifgt(offset) => Self::IfGt(target(*offset)),
            Opcode::IfIcmpgt(offset) => Self::IfICmpGt(target(*offset)),
            Opcode::Ifle(offset) => Self::IfLe(target(*offset)),
            Opcode::IfIcmple(offset) => Self::IfICmpLe(target(*offset)),
            Opcode::IfAcmpeq(offset) => Self::IfACmpEq(target(*offset)),
            Opcode::IfAcmpne(offset) => Self::IfACmpNe(target(*offset)),
            Opcode::Ifnull(offset) => Self::IfNull(target(*offset)),
            Opcode::Ifnonnull(offset) => Self::IfNonNull(target(*offset)),
            Opcode::Goto(offset) => Self::Goto(target(*offset)),
            Opcode::Ireturn => Self::IReturn,
            Opcode::Lreturn => Self::LReturn,
            Opcode::Freturn => Self::FReturn,
            Opcode::Dreturn => Self::DReturn,
            Opcode::Areturn => Self::AReturn,
            Opcode::Return => Self::Return,
            _ => Self::Unsupported,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_branches_use_byte_addresses_not_instruction_indices() {
        let code = ByteCode {
            opcodes: vec![
                (0, Opcode::Bipush(10)),
                (2, Opcode::Istore(0)),
                (3, Opcode::Goto(5)),
                (6, Opcode::Bipush(20)),
                (8, Opcode::Iload(0)),
                (9, Opcode::Ifne(-9)),
                (12, Opcode::Ireturn),
            ],
        };
        assert!(matches!(
            Instruction::lower(&code, 2),
            Instruction::Goto(InstIdx(4))
        ));
        assert!(matches!(
            Instruction::lower(&code, 5),
            Instruction::IfNe(InstIdx(0))
        ));
        // goto_w is normalized by cafebabe to the same Opcode::Goto variant.
        let wide = ByteCode {
            opcodes: vec![(0, Opcode::Goto(40_000)), (40_000, Opcode::Return)],
        };
        assert!(matches!(
            Instruction::lower(&wide, 0),
            Instruction::Goto(InstIdx(1))
        ));
    }

    #[test]
    fn lower_retains_wide_local_indices_and_signed_increments() {
        let code = ByteCode {
            opcodes: vec![
                (0, Opcode::Iinc(300, -1000)),
                (6, Opcode::Iload(300)),
                (10, Opcode::Lstore(400)),
            ],
        };
        assert!(matches!(
            Instruction::lower(&code, 0),
            Instruction::IInc(LocalIdx(300), -1000)
        ));
        assert!(matches!(
            Instruction::lower(&code, 1),
            Instruction::ILoad(LocalIdx(300))
        ));
        assert!(matches!(
            Instruction::lower(&code, 2),
            Instruction::LStore(LocalIdx(400))
        ));
    }

    #[test]
    fn lower_stack_variants_without_losing_their_forms() {
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
        let code = ByteCode {
            opcodes: opcodes.into_iter().enumerate().collect(),
        };
        for (idx, expected) in expected.iter().enumerate() {
            assert_eq!(
                std::mem::discriminant(&Instruction::lower(&code, idx)),
                std::mem::discriminant(expected)
            );
        }
    }
}
