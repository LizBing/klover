//! JVM bytecode decoding and structural validation.

use std::sync::Arc;

use super::byte_reader::ByteReader;
use super::constant_pool::{
    ClassMemberRef, ClassRef, ConstantPool, InvokeDynamicConstant, LoadableConstant, MemberKind,
};
use super::entries::{ResolvedPool, class_ref, invoke_dynamic, loadable, member, method_member};
use super::error::{ParseError, ParseErrorKind, ParseResult};

#[derive(Debug, Clone)]
pub(crate) struct BytecodeRangeTable {
    pub(crate) default: i32,
    pub(crate) low: i32,
    pub(crate) high: i32,
    pub(crate) jumps: Vec<i32>,
}

#[derive(Debug, Clone)]
pub(crate) struct BytecodeLookupTable {
    pub(crate) default: i32,
    pub(crate) match_offsets: Vec<(i32, i32)>,
}

#[derive(Debug, Clone)]
pub(crate) enum Opcode {
    Nop,
    AconstNull,
    Iconst(i32),
    Lconst(i64),
    Fconst(f32),
    Dconst(f64),
    Bipush(i8),
    Sipush(i16),
    Ldc(Arc<LoadableConstant>),
    LdcW(Arc<LoadableConstant>),
    Ldc2W(Arc<LoadableConstant>),

    Iload(u16),
    Lload(u16),
    Fload(u16),
    Dload(u16),
    Aload(u16),
    Istore(u16),
    Lstore(u16),
    Fstore(u16),
    Dstore(u16),
    Astore(u16),

    Pop,
    Pop2,
    Dup,
    DupX1,
    DupX2,
    Dup2,
    Dup2X1,
    Dup2X2,
    Swap,

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
    IInc(u16, i16),

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

    LCmp,
    FCmpL,
    FCmpG,
    DCmpL,
    DCmpG,
    IfEq(i32),
    IfNe(i32),
    IfLt(i32),
    IfGe(i32),
    IfGt(i32),
    IfLe(i32),
    IfICmpEq(i32),
    IfICmpNe(i32),
    IfICmpLt(i32),
    IfICmpGe(i32),
    IfICmpGt(i32),
    IfICmpLe(i32),
    IfACmpEq(i32),
    IfACmpNe(i32),

    Goto(i32),
    TableSwitch(BytecodeRangeTable),
    LookupSwitch(BytecodeLookupTable),

    IReturn,
    LReturn,
    FReturn,
    DReturn,
    AReturn,
    Return,

    IfNull(i32),
    IfNonNull(i32),

    GetStatic(Arc<ClassMemberRef>),
    PutStatic(Arc<ClassMemberRef>),
    GetField(Arc<ClassMemberRef>),
    PutField(Arc<ClassMemberRef>),
    InvokeVirtual(Arc<ClassMemberRef>),
    InvokeSpecial(Arc<ClassMemberRef>),
    InvokeStatic(Arc<ClassMemberRef>),
    InvokeInterface {
        method: Arc<ClassMemberRef>,
        count: u8,
    },
    InvokeDynamic(Arc<InvokeDynamicConstant>),

    New(Arc<ClassRef>),
    Anewarray(Arc<ClassRef>),
    Checkcast(Arc<ClassRef>),
    Instanceof(Arc<ClassRef>),
    Multianewarray {
        class: Arc<ClassRef>,
        dimensions: u8,
    },

    Unsupported(u8),
}

#[derive(Debug, Clone)]
pub(crate) struct Bytecode {
    pub(crate) opcodes: Vec<(u32, Opcode)>,
}

impl Bytecode {
    pub(crate) fn get_opcode_index(&self, bci: u32) -> Option<usize> {
        self.opcodes
            .binary_search_by_key(&bci, |(address, _)| *address)
            .ok()
    }
}

pub(crate) fn decode(
    code: &[u8],
    max_locals: u16,
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
) -> ParseResult<Bytecode> {
    let mut reader = ByteReader::new(code);
    let mut opcodes = Vec::new();

    while !reader.is_empty() {
        let pc = reader.pos() as u32;
        let opcode = decode_one(&mut reader, pc, max_locals, pool, resolved)?;
        opcodes.push((pc, opcode));
    }

    let bytecode = Bytecode { opcodes };
    validate_targets(&bytecode)?;
    Ok(bytecode)
}

fn decode_one(
    reader: &mut ByteReader<'_>,
    pc: u32,
    max_locals: u16,
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
) -> ParseResult<Opcode> {
    let byte = reader.read_u8()?;

    let opcode = match byte {
        0x00 => Opcode::Nop,
        0x01 => Opcode::AconstNull,
        0x02 => Opcode::Iconst(-1),
        0x03 => Opcode::Iconst(0),
        0x04 => Opcode::Iconst(1),
        0x05 => Opcode::Iconst(2),
        0x06 => Opcode::Iconst(3),
        0x07 => Opcode::Iconst(4),
        0x08 => Opcode::Iconst(5),
        0x09 => Opcode::Lconst(0),
        0x0a => Opcode::Lconst(1),
        0x0b => Opcode::Fconst(0.0),
        0x0c => Opcode::Fconst(1.0),
        0x0d => Opcode::Fconst(2.0),
        0x0e => Opcode::Dconst(0.0),
        0x0f => Opcode::Dconst(1.0),
        0x10 => Opcode::Bipush(reader.read_i8()?),
        0x11 => Opcode::Sipush(reader.read_i16()?),
        0x12 => {
            let index = reader.read_u8()? as u16;
            Opcode::Ldc(loadable(pool, index, false, pc, resolved)?)
        }
        0x13 => {
            let index = reader.read_u16()?;
            Opcode::LdcW(loadable(pool, index, false, pc, resolved)?)
        }
        0x14 => {
            let index = reader.read_u16()?;
            Opcode::Ldc2W(loadable(pool, index, true, pc, resolved)?)
        }
        0x15 => Opcode::Iload(local(reader, max_locals, pc)?),
        0x16 => Opcode::Lload(local(reader, max_locals, pc)?),
        0x17 => Opcode::Fload(local(reader, max_locals, pc)?),
        0x18 => Opcode::Dload(local(reader, max_locals, pc)?),
        0x19 => Opcode::Aload(local(reader, max_locals, pc)?),
        0x1a..=0x1d => Opcode::Iload(implicit_local(byte - 0x1a, max_locals, pc)?),
        0x1e..=0x21 => Opcode::Lload(implicit_local(byte - 0x1e, max_locals, pc)?),
        0x22..=0x25 => Opcode::Fload(implicit_local(byte - 0x22, max_locals, pc)?),
        0x26..=0x29 => Opcode::Dload(implicit_local(byte - 0x26, max_locals, pc)?),
        0x2a..=0x2d => Opcode::Aload(implicit_local(byte - 0x2a, max_locals, pc)?),
        0x36 => Opcode::Istore(local(reader, max_locals, pc)?),
        0x37 => Opcode::Lstore(local(reader, max_locals, pc)?),
        0x38 => Opcode::Fstore(local(reader, max_locals, pc)?),
        0x39 => Opcode::Dstore(local(reader, max_locals, pc)?),
        0x3a => Opcode::Astore(local(reader, max_locals, pc)?),
        0x3b..=0x3e => Opcode::Istore(implicit_local(byte - 0x3b, max_locals, pc)?),
        0x3f..=0x42 => Opcode::Lstore(implicit_local(byte - 0x3f, max_locals, pc)?),
        0x43..=0x46 => Opcode::Fstore(implicit_local(byte - 0x43, max_locals, pc)?),
        0x47..=0x4a => Opcode::Dstore(implicit_local(byte - 0x47, max_locals, pc)?),
        0x4b..=0x4e => Opcode::Astore(implicit_local(byte - 0x4b, max_locals, pc)?),
        0x57 => Opcode::Pop,
        0x58 => Opcode::Pop2,
        0x59 => Opcode::Dup,
        0x5a => Opcode::DupX1,
        0x5b => Opcode::DupX2,
        0x5c => Opcode::Dup2,
        0x5d => Opcode::Dup2X1,
        0x5e => Opcode::Dup2X2,
        0x5f => Opcode::Swap,
        0x60 => Opcode::IAdd,
        0x61 => Opcode::LAdd,
        0x62 => Opcode::FAdd,
        0x63 => Opcode::DAdd,
        0x64 => Opcode::ISub,
        0x65 => Opcode::LSub,
        0x66 => Opcode::FSub,
        0x67 => Opcode::DSub,
        0x68 => Opcode::IMul,
        0x69 => Opcode::LMul,
        0x6a => Opcode::FMul,
        0x6b => Opcode::DMul,
        0x6c => Opcode::IDiv,
        0x6d => Opcode::LDiv,
        0x6e => Opcode::FDiv,
        0x6f => Opcode::DDiv,
        0x70 => Opcode::IRem,
        0x71 => Opcode::LRem,
        0x72 => Opcode::FRem,
        0x73 => Opcode::DRem,
        0x74 => Opcode::INeg,
        0x75 => Opcode::LNeg,
        0x76 => Opcode::FNeg,
        0x77 => Opcode::DNeg,
        0x78 => Opcode::IShL,
        0x79 => Opcode::LShL,
        0x7a => Opcode::IShR,
        0x7b => Opcode::LShR,
        0x7c => Opcode::IUShR,
        0x7d => Opcode::LUShR,
        0x7e => Opcode::IAnd,
        0x7f => Opcode::LAnd,
        0x80 => Opcode::IOr,
        0x81 => Opcode::LOr,
        0x82 => Opcode::IXor,
        0x83 => Opcode::LXor,
        0x84 => {
            let index = local(reader, max_locals, pc)?;
            let amount = reader.read_i8()? as i16;
            Opcode::IInc(index, amount)
        }
        0x85 => Opcode::I2L,
        0x86 => Opcode::I2F,
        0x87 => Opcode::I2D,
        0x88 => Opcode::L2I,
        0x89 => Opcode::L2F,
        0x8a => Opcode::L2D,
        0x8b => Opcode::F2I,
        0x8c => Opcode::F2L,
        0x8d => Opcode::F2D,
        0x8e => Opcode::D2I,
        0x8f => Opcode::D2L,
        0x90 => Opcode::D2F,
        0x91 => Opcode::I2B,
        0x92 => Opcode::I2C,
        0x93 => Opcode::I2S,
        0x94 => Opcode::LCmp,
        0x95 => Opcode::FCmpL,
        0x96 => Opcode::FCmpG,
        0x97 => Opcode::DCmpL,
        0x98 => Opcode::DCmpG,
        0x99 => Opcode::IfEq(reader.read_i16()? as i32),
        0x9a => Opcode::IfNe(reader.read_i16()? as i32),
        0x9b => Opcode::IfLt(reader.read_i16()? as i32),
        0x9c => Opcode::IfGe(reader.read_i16()? as i32),
        0x9d => Opcode::IfGt(reader.read_i16()? as i32),
        0x9e => Opcode::IfLe(reader.read_i16()? as i32),
        0x9f => Opcode::IfICmpEq(reader.read_i16()? as i32),
        0xa0 => Opcode::IfICmpNe(reader.read_i16()? as i32),
        0xa1 => Opcode::IfICmpLt(reader.read_i16()? as i32),
        0xa2 => Opcode::IfICmpGe(reader.read_i16()? as i32),
        0xa3 => Opcode::IfICmpGt(reader.read_i16()? as i32),
        0xa4 => Opcode::IfICmpLe(reader.read_i16()? as i32),
        0xa5 => Opcode::IfACmpEq(reader.read_i16()? as i32),
        0xa6 => Opcode::IfACmpNe(reader.read_i16()? as i32),
        0xa7 => Opcode::Goto(reader.read_i16()? as i32),
        0xaa => decode_tableswitch(reader, pc)?,
        0xab => decode_lookupswitch(reader, pc)?,
        0xac => Opcode::IReturn,
        0xad => Opcode::LReturn,
        0xae => Opcode::FReturn,
        0xaf => Opcode::DReturn,
        0xb0 => Opcode::AReturn,
        0xb1 => Opcode::Return,
        0xb2 => {
            let index = reader.read_u16()?;
            Opcode::GetStatic(member(pool, index, MemberKind::Field, pc, resolved)?)
        }
        0xb3 => {
            let index = reader.read_u16()?;
            Opcode::PutStatic(member(pool, index, MemberKind::Field, pc, resolved)?)
        }
        0xb4 => {
            let index = reader.read_u16()?;
            Opcode::GetField(member(pool, index, MemberKind::Field, pc, resolved)?)
        }
        0xb5 => {
            let index = reader.read_u16()?;
            Opcode::PutField(member(pool, index, MemberKind::Field, pc, resolved)?)
        }
        0xb6 => {
            let index = reader.read_u16()?;
            Opcode::InvokeVirtual(member(pool, index, MemberKind::Method, pc, resolved)?)
        }
        0xb7 => {
            let index = reader.read_u16()?;
            Opcode::InvokeSpecial(method_member(pool, index, pc, resolved)?)
        }
        0xb8 => {
            let index = reader.read_u16()?;
            Opcode::InvokeStatic(method_member(pool, index, pc, resolved)?)
        }
        0xb9 => {
            let index = reader.read_u16()?;
            let count = reader.read_u8()?;
            let zero = reader.read_u8()?;
            if count == 0 || zero != 0 {
                return Err(invalid(pc, ParseErrorKind::InvalidInstruction(0xb9)));
            }
            Opcode::InvokeInterface {
                method: member(pool, index, MemberKind::InterfaceMethod, pc, resolved)?,
                count,
            }
        }
        0xba => {
            let index = reader.read_u16()?;
            let zero = reader.read_u16()?;
            if zero != 0 {
                return Err(invalid(pc, ParseErrorKind::InvalidInstruction(0xba)));
            }
            Opcode::InvokeDynamic(invoke_dynamic(pool, index, pc, resolved)?)
        }
        0xbb => {
            let index = reader.read_u16()?;
            Opcode::New(class_ref(pool, index, pc, resolved)?)
        }
        0xbc => {
            let atype = reader.read_u8()?;
            if !(4..=11).contains(&atype) {
                return Err(invalid(pc, ParseErrorKind::InvalidInstruction(0xbc)));
            }
            Opcode::Unsupported(0xbc)
        }
        0xbd => {
            let index = reader.read_u16()?;
            Opcode::Anewarray(class_ref(pool, index, pc, resolved)?)
        }
        0xbe | 0xbf | 0xc2 | 0xc3 => Opcode::Unsupported(byte),
        0xc0 => {
            let index = reader.read_u16()?;
            Opcode::Checkcast(class_ref(pool, index, pc, resolved)?)
        }
        0xc1 => {
            let index = reader.read_u16()?;
            Opcode::Instanceof(class_ref(pool, index, pc, resolved)?)
        }
        0xc4 => decode_wide(reader, pc, max_locals)?,
        0xc5 => {
            let index = reader.read_u16()?;
            let dimensions = reader.read_u8()?;
            if dimensions == 0 {
                return Err(invalid(pc, ParseErrorKind::InvalidInstruction(0xc5)));
            }
            Opcode::Multianewarray {
                class: class_ref(pool, index, pc, resolved)?,
                dimensions,
            }
        }
        0xc6 => Opcode::IfNull(reader.read_i16()? as i32),
        0xc7 => Opcode::IfNonNull(reader.read_i16()? as i32),
        0xc8 => Opcode::Goto(reader.read_i32()?),
        0x2e..=0x35 | 0x4f..=0x56 => Opcode::Unsupported(byte),
        0xa8 | 0xa9 | 0xc9 | 0xca | 0xfe | 0xff => {
            return Err(invalid(pc, ParseErrorKind::InvalidInstruction(byte)));
        }
        _ => return Err(invalid(pc, ParseErrorKind::InvalidInstruction(byte))),
    };

    Ok(opcode)
}

fn local(reader: &mut ByteReader<'_>, max_locals: u16, pc: u32) -> ParseResult<u16> {
    let index = reader.read_u8()? as u16;
    if index >= max_locals {
        return Err(invalid(
            pc,
            ParseErrorKind::InvalidLocalIndex { index, max_locals },
        ));
    }
    Ok(index)
}

fn implicit_local(index: u8, max_locals: u16, pc: u32) -> ParseResult<u16> {
    let index = index as u16;
    if index >= max_locals {
        return Err(invalid(
            pc,
            ParseErrorKind::InvalidLocalIndex { index, max_locals },
        ));
    }
    Ok(index)
}

fn wide_local(reader: &mut ByteReader<'_>, max_locals: u16, pc: u32) -> ParseResult<u16> {
    let index = reader.read_u16()?;
    if index >= max_locals {
        return Err(invalid(
            pc,
            ParseErrorKind::InvalidLocalIndex { index, max_locals },
        ));
    }
    Ok(index)
}

fn decode_wide(reader: &mut ByteReader<'_>, pc: u32, max_locals: u16) -> ParseResult<Opcode> {
    let modified = reader.read_u8()?;
    let opcode = match modified {
        0x15 => Opcode::Iload(wide_local(reader, max_locals, pc)?),
        0x16 => Opcode::Lload(wide_local(reader, max_locals, pc)?),
        0x17 => Opcode::Fload(wide_local(reader, max_locals, pc)?),
        0x18 => Opcode::Dload(wide_local(reader, max_locals, pc)?),
        0x19 => Opcode::Aload(wide_local(reader, max_locals, pc)?),
        0x36 => Opcode::Istore(wide_local(reader, max_locals, pc)?),
        0x37 => Opcode::Lstore(wide_local(reader, max_locals, pc)?),
        0x38 => Opcode::Fstore(wide_local(reader, max_locals, pc)?),
        0x39 => Opcode::Dstore(wide_local(reader, max_locals, pc)?),
        0x3a => Opcode::Astore(wide_local(reader, max_locals, pc)?),
        0x84 => {
            let index = wide_local(reader, max_locals, pc)?;
            let amount = reader.read_i16()?;
            Opcode::IInc(index, amount)
        }
        _ => return Err(invalid(pc, ParseErrorKind::InvalidInstruction(0xc4))),
    };
    Ok(opcode)
}

fn decode_tableswitch(reader: &mut ByteReader<'_>, pc: u32) -> ParseResult<Opcode> {
    skip_switch_padding(reader, pc)?;
    let default = reader.read_i32()?;
    let low = reader.read_i32()?;
    let high = reader.read_i32()?;
    if high < low {
        return Err(invalid(pc, ParseErrorKind::InvalidSwitch));
    }
    let count = i64::from(high) - i64::from(low) + 1;
    let count = usize::try_from(count).map_err(|_| invalid(pc, ParseErrorKind::InvalidSwitch))?;
    if count > reader.remaining() / 4 {
        return Err(invalid(pc, ParseErrorKind::InvalidSwitch));
    }
    let mut jumps = Vec::with_capacity(count);
    for _ in 0..count {
        jumps.push(reader.read_i32()?);
    }
    Ok(Opcode::TableSwitch(BytecodeRangeTable {
        default,
        low,
        high,
        jumps,
    }))
}

fn decode_lookupswitch(reader: &mut ByteReader<'_>, pc: u32) -> ParseResult<Opcode> {
    skip_switch_padding(reader, pc)?;
    let default = reader.read_i32()?;
    let pairs = reader.read_i32()?;
    if pairs < 0 {
        return Err(invalid(pc, ParseErrorKind::InvalidSwitch));
    }
    let pairs = pairs as usize;
    if pairs > reader.remaining() / 8 {
        return Err(invalid(pc, ParseErrorKind::InvalidSwitch));
    }
    let mut match_offsets = Vec::with_capacity(pairs);
    let mut previous: Option<i32> = None;
    for _ in 0..pairs {
        let key = reader.read_i32()?;
        if let Some(previous) = previous {
            if key <= previous {
                return Err(invalid(pc, ParseErrorKind::InvalidSwitch));
            }
        }
        previous = Some(key);
        let offset = reader.read_i32()?;
        match_offsets.push((key, offset));
    }
    Ok(Opcode::LookupSwitch(BytecodeLookupTable {
        default,
        match_offsets,
    }))
}

fn skip_switch_padding(reader: &mut ByteReader<'_>, pc: u32) -> ParseResult<()> {
    let padding = (4 - ((pc as usize + 1) & 3)) & 3;
    reader.skip(padding)
}

fn validate_targets(bytecode: &Bytecode) -> ParseResult<()> {
    for &(pc, ref opcode) in &bytecode.opcodes {
        match opcode {
            Opcode::IfEq(offset)
            | Opcode::IfNe(offset)
            | Opcode::IfLt(offset)
            | Opcode::IfGe(offset)
            | Opcode::IfGt(offset)
            | Opcode::IfLe(offset)
            | Opcode::IfICmpEq(offset)
            | Opcode::IfICmpNe(offset)
            | Opcode::IfICmpLt(offset)
            | Opcode::IfICmpGe(offset)
            | Opcode::IfICmpGt(offset)
            | Opcode::IfICmpLe(offset)
            | Opcode::IfACmpEq(offset)
            | Opcode::IfACmpNe(offset)
            | Opcode::Goto(offset)
            | Opcode::IfNull(offset)
            | Opcode::IfNonNull(offset) => validate_target(bytecode, pc, *offset)?,
            Opcode::TableSwitch(table) => {
                validate_target(bytecode, pc, table.default)?;
                for &offset in &table.jumps {
                    validate_target(bytecode, pc, offset)?;
                }
            }
            Opcode::LookupSwitch(table) => {
                validate_target(bytecode, pc, table.default)?;
                for &(_, offset) in &table.match_offsets {
                    validate_target(bytecode, pc, offset)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_target(bytecode: &Bytecode, pc: u32, offset: i32) -> ParseResult<()> {
    let target = i64::from(pc) + i64::from(offset);
    if target < 0 || target > i64::from(u32::MAX) {
        return Err(invalid(
            pc,
            ParseErrorKind::InvalidBranchTarget { bci: pc, target },
        ));
    }
    if bytecode.get_opcode_index(target as u32).is_none() {
        return Err(invalid(
            pc,
            ParseErrorKind::InvalidBranchTarget { bci: pc, target },
        ));
    }
    Ok(())
}

fn invalid(pc: u32, kind: ParseErrorKind) -> ParseError {
    ParseError::new(pc as usize, kind)
}

#[cfg(test)]
mod tests {
    use super::super::constant_pool::Constant;
    use super::*;

    #[test]
    fn accepts_instruction_boundaries_and_rejects_bad_targets() {
        let pool = ConstantPool::empty();
        let mut resolved = ResolvedPool::default();
        let valid = [0xa7, 0x00, 0x03, 0xb1];
        let decoded = decode(&valid, 0, &pool, &mut resolved).unwrap();
        assert_eq!(decoded.opcodes.len(), 2);

        let bad = [0xa7, 0x00, 0x05, 0xb1, 0x00];
        assert!(decode(&bad, 0, &pool, &mut resolved).is_err());
    }

    #[test]
    fn rejects_local_indices_beyond_max_locals() {
        let pool = ConstantPool::empty();
        let mut resolved = ResolvedPool::default();
        assert!(decode(&[0x15, 0x00, 0xb1], 0, &pool, &mut resolved).is_err());
        assert!(decode(&[0x15, 0x00, 0xb1], 1, &pool, &mut resolved).is_ok());
    }

    #[test]
    fn decodes_tableswitch_and_lookupswitch() {
        let pool = ConstantPool::empty();
        let mut resolved = ResolvedPool::default();
        let code = [
            0xaa, 0x00, 0x00, 0x00, // tableswitch at 0, padding
            0x00, 0x00, 0x00, 0x14, // default +20 -> bci 20
            0x00, 0x00, 0x00, 0x00, // low 0
            0x00, 0x00, 0x00, 0x00, // high 0
            0x00, 0x00, 0x00, 0x14, // case 0 -> bci 20
            0xb1, // bci 20
        ];
        assert!(decode(&code, 0, &pool, &mut resolved).is_ok());

        let lookup = [
            0xab, 0x00, 0x00, 0x00, // lookupswitch at 0, padding
            0x00, 0x00, 0x00, 0x14, // default +20 -> bci 20
            0x00, 0x00, 0x00, 0x01, // one pair
            0x00, 0x00, 0x00, 0x07, // key 7
            0x00, 0x00, 0x00, 0x14, // offset +20
            0xb1, // bci 20
        ];
        assert!(decode(&lookup, 0, &pool, &mut resolved).is_ok());
    }

    #[test]
    fn shares_resolved_entries_across_instructions() {
        use crate::class_parser::JavaText;

        let pool = ConstantPool::new(vec![
            Constant::Unusable,
            Constant::Utf8(JavaText::from_str("java/lang/Object")),
            Constant::Class(1),
            Constant::Utf8(JavaText::from_str("m")),
            Constant::Utf8(JavaText::from_str("()V")),
            Constant::NameAndType {
                name: 3,
                descriptor: 4,
            },
            Constant::MethodRef {
                owner: 2,
                name_and_type: 5,
            },
            Constant::Integer(42),
        ]);
        let mut resolved = ResolvedPool::default();
        let code = [
            0x12, 0x07, // ldc #7
            0x12, 0x07, // ldc #7
            0xb8, 0x00, 0x06, // invokestatic #6
            0xb8, 0x00, 0x06, // invokestatic #6
            0xb1,
        ];
        let decoded = decode(&code, 0, &pool, &mut resolved).unwrap();

        let Opcode::Ldc(first) = &decoded.opcodes[0].1 else {
            panic!("expected ldc");
        };
        let Opcode::Ldc(second) = &decoded.opcodes[1].1 else {
            panic!("expected ldc");
        };
        assert!(Arc::ptr_eq(first, second));

        let Opcode::InvokeStatic(first) = &decoded.opcodes[2].1 else {
            panic!("expected invokestatic");
        };
        let Opcode::InvokeStatic(second) = &decoded.opcodes[3].1 else {
            panic!("expected invokestatic");
        };
        assert!(Arc::ptr_eq(first, second));
        assert_eq!(first.owner.name.as_str(), "java/lang/Object");
        assert_eq!(first.name.as_str(), "m");
        assert_eq!(first.descriptor.as_str(), "()V");
    }
}
