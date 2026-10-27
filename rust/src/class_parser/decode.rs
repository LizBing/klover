//! Framing-only class-file decoding. Semantic checks happen in `validate`.

use super::byte_reader::ByteReader;
use super::error::{ParseError, ParseErrorKind, ParseResult};

pub(super) struct RawClassImage<'a> {
    pub(super) minor: u16,
    pub(super) major: u16,
    pub(super) access_flags: u16,
    pub(super) constant_pool: Vec<RawConstant<'a>>,
    pub(super) this_class: u16,
    pub(super) super_class: Option<u16>,
    pub(super) interfaces: Vec<u16>,
    pub(super) fields: Vec<RawMember<'a>>,
    pub(super) methods: Vec<RawMember<'a>>,
    pub(super) attributes: Vec<RawAttribute<'a>>,
}

pub(super) enum RawConstant<'a> {
    Unusable,
    Utf8 { bytes: &'a [u8], offset: usize },
    Integer(i32),
    Float(u32),
    Long(i64),
    Double(u64),
    Class(u16),
    String(u16),
    FieldRef { owner: u16, name_and_type: u16 },
    MethodRef { owner: u16, name_and_type: u16 },
    InterfaceMethodRef { owner: u16, name_and_type: u16 },
    NameAndType { name: u16, descriptor: u16 },
    MethodHandle { kind: u8, reference: u16 },
    MethodType(u16),
    InvokeDynamic { bootstrap: u16, name_and_type: u16 },
}

pub(super) struct RawMember<'a> {
    pub(super) access_flags: u16,
    pub(super) name_index: u16,
    pub(super) descriptor_index: u16,
    pub(super) attributes: Vec<RawAttribute<'a>>,
}

pub(super) struct RawAttribute<'a> {
    pub(super) name_index: u16,
    pub(super) body: &'a [u8],
}

pub(super) fn decode(bytes: &[u8]) -> ParseResult<RawClassImage<'_>> {
    let mut reader = ByteReader::new(bytes);

    if reader.read_u32()? != 0xCAFE_BABE {
        return Err(ParseError::new(0, ParseErrorKind::BadMagic));
    }

    let minor = reader.read_u16()?;
    let major = reader.read_u16()?;
    if major != 52 {
        return Err(ParseError::new(
            4,
            ParseErrorKind::UnsupportedVersion { major, minor },
        ));
    }

    let constant_pool = read_constant_pool(&mut reader)?;
    let access_flags = reader.read_u16()?;
    let this_class = reader.read_u16()?;
    let super_class = match reader.read_u16()? {
        0 => None,
        index => Some(index),
    };
    let interfaces = read_u16_vec(&mut reader)?;
    let fields = read_members(&mut reader)?;
    let methods = read_members(&mut reader)?;
    let attributes = read_attributes(&mut reader)?;

    if !reader.is_empty() {
        return Err(ParseError::new(reader.pos(), ParseErrorKind::TrailingBytes));
    }

    Ok(RawClassImage {
        minor,
        major,
        access_flags,
        constant_pool,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
    })
}

fn read_constant_pool<'a>(reader: &mut ByteReader<'a>) -> ParseResult<Vec<RawConstant<'a>>> {
    let count = reader.read_u16()?;
    if count == 0 {
        return Err(ParseError::new(
            reader.pos(),
            ParseErrorKind::BadConstantPoolEntry("constant_pool_count is zero"),
        ));
    }

    let mut pool = Vec::with_capacity(count as usize);
    pool.push(RawConstant::Unusable);

    while pool.len() < count as usize {
        let offset = reader.pos();
        let tag = reader.read_u8()?;
        let entry = match tag {
            1 => {
                let len = reader.read_u16()? as usize;
                RawConstant::Utf8 {
                    bytes: reader.read_bytes(len)?,
                    offset,
                }
            }
            3 => RawConstant::Integer(reader.read_i32()?),
            4 => RawConstant::Float(reader.read_u32()?),
            5 => {
                let value = RawConstant::Long(reader.read_i64()?);
                pool.push(value);
                pool.push(RawConstant::Unusable);
                continue;
            }
            6 => {
                let value = RawConstant::Double(reader.read_u64()?);
                pool.push(value);
                pool.push(RawConstant::Unusable);
                continue;
            }
            7 => RawConstant::Class(reader.read_u16()?),
            8 => RawConstant::String(reader.read_u16()?),
            9 => RawConstant::FieldRef {
                owner: reader.read_u16()?,
                name_and_type: reader.read_u16()?,
            },
            10 => RawConstant::MethodRef {
                owner: reader.read_u16()?,
                name_and_type: reader.read_u16()?,
            },
            11 => RawConstant::InterfaceMethodRef {
                owner: reader.read_u16()?,
                name_and_type: reader.read_u16()?,
            },
            12 => RawConstant::NameAndType {
                name: reader.read_u16()?,
                descriptor: reader.read_u16()?,
            },
            15 => RawConstant::MethodHandle {
                kind: reader.read_u8()?,
                reference: reader.read_u16()?,
            },
            16 => RawConstant::MethodType(reader.read_u16()?),
            18 => RawConstant::InvokeDynamic {
                bootstrap: reader.read_u16()?,
                name_and_type: reader.read_u16()?,
            },
            other => {
                return Err(ParseError::new(
                    offset,
                    ParseErrorKind::BadConstantPoolTag(other),
                ));
            }
        };
        pool.push(entry);
    }

    if pool.len() != count as usize {
        return Err(ParseError::new(
            reader.pos(),
            ParseErrorKind::BadConstantPoolEntry("constant pool count mismatch"),
        ));
    }

    Ok(pool)
}

fn read_u16_vec(reader: &mut ByteReader<'_>) -> ParseResult<Vec<u16>> {
    let count = reader.read_u16()? as usize;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(reader.read_u16()?);
    }
    Ok(values)
}

fn read_members<'a>(reader: &mut ByteReader<'a>) -> ParseResult<Vec<RawMember<'a>>> {
    let count = reader.read_u16()? as usize;
    let mut members = Vec::with_capacity(count);
    for _ in 0..count {
        members.push(RawMember {
            access_flags: reader.read_u16()?,
            name_index: reader.read_u16()?,
            descriptor_index: reader.read_u16()?,
            attributes: read_attributes(reader)?,
        });
    }
    Ok(members)
}

pub(super) fn read_attributes<'a>(
    reader: &mut ByteReader<'a>,
) -> ParseResult<Vec<RawAttribute<'a>>> {
    let count = reader.read_u16()? as usize;
    let mut attributes = Vec::with_capacity(count);
    for _ in 0..count {
        let name_index = reader.read_u16()?;
        let len = reader.read_u32()? as usize;
        attributes.push(RawAttribute {
            name_index,
            body: reader.read_bytes(len)?,
        });
    }
    Ok(attributes)
}
