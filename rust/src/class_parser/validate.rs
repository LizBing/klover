//! Semantic validation and conversion to the published class image.

use std::collections::HashSet;

use super::byte_reader::ByteReader;
use super::bytecode::ResolvedPool;
use super::constant_pool::{Constant, ConstantPool};
use super::decode::{RawAttribute, RawClassImage, RawConstant, RawMember};
use super::descriptor::{FieldDescriptor, MethodDescriptor};
use super::error::{ParseError, ParseErrorKind, ParseResult};
use super::flags::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use super::model::{
    ClassData, CodeData, ExceptionHandler, FieldInfo, MethodInfo, ValidatedClassImage,
};
use super::text::JavaText;

pub(super) fn validate(raw: RawClassImage<'_>) -> ParseResult<ValidatedClassImage> {
    let RawClassImage {
        minor,
        major,
        access_flags,
        constant_pool,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
    } = raw;

    let pool = build_pool(constant_pool)?;
    validate_pool(&pool)?;

    let this_class = class_name(&pool, this_class)?;
    validate_class_name(&this_class)?;
    let super_class = match super_class {
        Some(index) => Some(class_name(&pool, index)?),
        None => None,
    };
    let interfaces = interfaces
        .into_iter()
        .map(|index| class_name(&pool, index))
        .collect::<ParseResult<Vec<_>>>()?;

    let fields = validate_fields(fields, &pool)?;
    let mut resolved = ResolvedPool::default();
    let methods = validate_methods(methods, &pool, &mut resolved)?;

    Ok(ValidatedClassImage::new(ClassData {
        minor,
        major,
        access_flags: ClassAccessFlags::from_bits_retain(access_flags),
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        constant_pool: pool,
    }))
}

fn build_pool(raw: Vec<RawConstant<'_>>) -> ParseResult<ConstantPool> {
    let mut entries = Vec::with_capacity(raw.len());
    for entry in raw {
        let entry = match entry {
            RawConstant::Unusable => Constant::Unusable,
            RawConstant::Utf8 { bytes, offset } => {
                Constant::Utf8(JavaText::from_modified_utf8(bytes, offset)?)
            }
            RawConstant::Integer(value) => Constant::Integer(value),
            RawConstant::Float(bits) => Constant::Float(bits),
            RawConstant::Long(value) => Constant::Long(value),
            RawConstant::Double(bits) => Constant::Double(bits),
            RawConstant::Class(name) => Constant::Class(name),
            RawConstant::String(text) => Constant::String(text),
            RawConstant::FieldRef {
                owner,
                name_and_type,
            } => Constant::FieldRef {
                owner,
                name_and_type,
            },
            RawConstant::MethodRef {
                owner,
                name_and_type,
            } => Constant::MethodRef {
                owner,
                name_and_type,
            },
            RawConstant::InterfaceMethodRef {
                owner,
                name_and_type,
            } => Constant::InterfaceMethodRef {
                owner,
                name_and_type,
            },
            RawConstant::NameAndType { name, descriptor } => {
                Constant::NameAndType { name, descriptor }
            }
            RawConstant::MethodHandle { kind, reference } => {
                Constant::MethodHandle { kind, reference }
            }
            RawConstant::MethodType(descriptor) => Constant::MethodType(descriptor),
            RawConstant::InvokeDynamic {
                bootstrap,
                name_and_type,
            } => Constant::InvokeDynamic {
                bootstrap,
                name_and_type,
            },
        };
        entries.push(entry);
    }
    Ok(ConstantPool::new(entries))
}

fn validate_pool(pool: &ConstantPool) -> ParseResult<()> {
    for index in 1..pool.len() {
        let index = index as u16;
        match pool.entry(index).expect("in-range constant") {
            Constant::Unusable
            | Constant::Utf8(_)
            | Constant::Integer(_)
            | Constant::Float(_)
            | Constant::Long(_)
            | Constant::Double(_) => {}
            Constant::Class(name) => {
                require_utf8(pool, *name)?;
            }
            Constant::String(text) => {
                require_utf8(pool, *text)?;
            }
            Constant::FieldRef {
                owner,
                name_and_type,
            } => {
                require_class(pool, *owner)?;
                let (name, descriptor) = require_name_and_type(pool, *name_and_type)?;
                let name = text_string(name)?;
                let descriptor = text_string(descriptor)?;
                validate_member_name(&name, false)?;
                FieldDescriptor::parse(&descriptor)?;
            }
            Constant::MethodRef {
                owner,
                name_and_type,
            }
            | Constant::InterfaceMethodRef {
                owner,
                name_and_type,
            } => {
                require_class(pool, *owner)?;
                let (name, descriptor) = require_name_and_type(pool, *name_and_type)?;
                let name = text_string(name)?;
                let descriptor = text_string(descriptor)?;
                validate_member_name(&name, true)?;
                MethodDescriptor::parse(&descriptor)?;
            }
            Constant::NameAndType { name, descriptor } => {
                require_utf8(pool, *name)?;
                require_utf8(pool, *descriptor)?;
            }
            Constant::MethodHandle { kind, reference } => {
                validate_method_handle(pool, *kind, *reference)?;
            }
            Constant::MethodType(descriptor) => {
                let descriptor = utf8_string(pool, *descriptor)?;
                MethodDescriptor::parse(&descriptor)?;
            }
            Constant::InvokeDynamic {
                bootstrap: _,
                name_and_type,
            } => {
                let (_, descriptor) = require_name_and_type(pool, *name_and_type)?;
                let descriptor = text_string(descriptor)?;
                MethodDescriptor::parse(&descriptor)?;
            }
        }
    }
    Ok(())
}

fn validate_method_handle(pool: &ConstantPool, kind: u8, reference: u16) -> ParseResult<()> {
    let allowed = match kind {
        1..=4 => matches!(pool.entry(reference), Some(Constant::FieldRef { .. })),
        5 | 8 => matches!(pool.entry(reference), Some(Constant::MethodRef { .. })),
        6 | 7 => matches!(
            pool.entry(reference),
            Some(Constant::MethodRef { .. }) | Some(Constant::InterfaceMethodRef { .. })
        ),
        9 => matches!(
            pool.entry(reference),
            Some(Constant::InterfaceMethodRef { .. })
        ),
        _ => false,
    };
    if !allowed {
        return Err(pool_error(ParseErrorKind::BadConstantPoolEntry(
            "invalid method-handle reference",
        )));
    }
    if kind == 8 {
        let Some(Constant::MethodRef { name_and_type, .. }) = pool.entry(reference) else {
            unreachable!("checked above");
        };
        let (name, _) = require_name_and_type(pool, *name_and_type)?;
        if !name.eq_str("<init>") {
            return Err(pool_error(ParseErrorKind::BadConstantPoolEntry(
                "newInvokeSpecial target must be <init>",
            )));
        }
    }
    Ok(())
}

fn validate_fields(raw: Vec<RawMember<'_>>, pool: &ConstantPool) -> ParseResult<Vec<FieldInfo>> {
    let mut seen = HashSet::new();
    let mut fields = Vec::with_capacity(raw.len());

    for member in raw {
        let name = utf8_string(pool, member.name_index)?;
        validate_member_name(&name, false)?;
        let descriptor = FieldDescriptor::parse(&utf8_string(pool, member.descriptor_index)?)?;
        validate_attribute_names(&member.attributes, pool)?;

        if !seen.insert((name.clone(), descriptor.raw().to_owned())) {
            return Err(pool_error(ParseErrorKind::DuplicateMember));
        }

        fields.push(FieldInfo {
            access_flags: FieldAccessFlags::from_bits_retain(member.access_flags),
            name,
            descriptor,
        });
    }

    Ok(fields)
}

fn validate_methods(
    raw: Vec<RawMember<'_>>,
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
) -> ParseResult<Vec<MethodInfo>> {
    let mut seen = HashSet::new();
    let mut methods = Vec::with_capacity(raw.len());

    for member in raw {
        let name = utf8_string(pool, member.name_index)?;
        validate_member_name(&name, true)?;
        let descriptor = MethodDescriptor::parse(&utf8_string(pool, member.descriptor_index)?)?;

        let mut code = None;
        for attribute in &member.attributes {
            let attribute_name = pool.utf8(attribute.name_index).ok_or_else(|| {
                ParseError::new(
                    0,
                    ParseErrorKind::BadConstantPoolIndex(attribute.name_index),
                )
            })?;
            if attribute_name.eq_str("Code") {
                if code.is_some() {
                    return Err(pool_error(ParseErrorKind::MultipleCodeAttributes));
                }
                code = Some(parse_code(attribute.body, pool, resolved)?);
            }
        }

        if !seen.insert((name.clone(), descriptor.raw().to_owned())) {
            return Err(pool_error(ParseErrorKind::DuplicateMember));
        }

        methods.push(MethodInfo {
            access_flags: MethodAccessFlags::from_bits_retain(member.access_flags),
            name,
            descriptor,
            code,
        });
    }

    Ok(methods)
}

fn parse_code(
    body: &[u8],
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
) -> ParseResult<CodeData> {
    let mut reader = ByteReader::new(body);
    let max_stack = reader.read_u16()?;
    let max_locals = reader.read_u16()?;
    let code_len = reader.read_u32()? as usize;
    if code_len == 0 || code_len > 65535 {
        return Err(pool_error(ParseErrorKind::InvalidCode(
            "code_length must be between 1 and 65535",
        )));
    }
    let code = reader.read_bytes(code_len)?;

    let exception_count = reader.read_u16()? as usize;
    let mut exception_table = Vec::with_capacity(exception_count);
    for _ in 0..exception_count {
        let start_pc = reader.read_u16()?;
        let end_pc = reader.read_u16()?;
        let handler_pc = reader.read_u16()?;
        let catch_type = reader.read_u16()?;
        if start_pc >= end_pc || end_pc as usize > code_len || handler_pc as usize >= code_len {
            return Err(pool_error(ParseErrorKind::InvalidExceptionTable));
        }
        if catch_type != 0 {
            require_class(pool, catch_type)?;
        }
        exception_table.push(ExceptionHandler {
            start_pc,
            end_pc,
            handler_pc,
            catch_type,
        });
    }

    skip_attributes(&mut reader, pool)?;
    if !reader.is_empty() {
        return Err(pool_error(ParseErrorKind::InvalidCode(
            "trailing bytes in Code attribute",
        )));
    }

    let bytecode = super::bytecode::decode(code, max_locals, pool, resolved)?;

    for handler in &exception_table {
        if bytecode.get_opcode_index(handler.start_pc as u32).is_none()
            || bytecode
                .get_opcode_index(handler.handler_pc as u32)
                .is_none()
            || (handler.end_pc as usize != code_len
                && bytecode.get_opcode_index(handler.end_pc as u32).is_none())
        {
            return Err(pool_error(ParseErrorKind::InvalidExceptionTable));
        }
    }

    Ok(CodeData {
        max_stack,
        max_locals,
        bytecode,
        exception_table,
    })
}

fn skip_attributes(reader: &mut ByteReader<'_>, pool: &ConstantPool) -> ParseResult<()> {
    let count = reader.read_u16()? as usize;
    for _ in 0..count {
        let name_index = reader.read_u16()?;
        if pool.utf8(name_index).is_none() {
            return Err(pool_error(ParseErrorKind::BadConstantPoolIndex(name_index)));
        }
        let len = reader.read_u32()? as usize;
        reader.skip(len)?;
    }
    Ok(())
}

fn validate_attribute_names(
    attributes: &[RawAttribute<'_>],
    pool: &ConstantPool,
) -> ParseResult<()> {
    for attribute in attributes {
        if pool.utf8(attribute.name_index).is_none() {
            return Err(pool_error(ParseErrorKind::BadConstantPoolIndex(
                attribute.name_index,
            )));
        }
    }
    Ok(())
}

fn require_utf8(pool: &ConstantPool, index: u16) -> ParseResult<&JavaText> {
    pool.utf8(index)
        .ok_or_else(|| pool_error(ParseErrorKind::BadConstantPoolIndex(index)))
}

fn require_class(pool: &ConstantPool, index: u16) -> ParseResult<()> {
    if pool.class_name(index).is_some() {
        Ok(())
    } else if pool.entry(index).is_none() {
        Err(pool_error(ParseErrorKind::BadConstantPoolIndex(index)))
    } else {
        Err(pool_error(ParseErrorKind::BadConstantPoolEntry(
            "expected class entry",
        )))
    }
}

fn require_name_and_type(pool: &ConstantPool, index: u16) -> ParseResult<(&JavaText, &JavaText)> {
    pool.name_and_type(index)
        .ok_or_else(|| pool_error(ParseErrorKind::BadConstantPoolIndex(index)))
}

fn utf8_string(pool: &ConstantPool, index: u16) -> ParseResult<String> {
    let text = require_utf8(pool, index)?;
    text_string(text)
}

fn text_string(text: &JavaText) -> ParseResult<String> {
    text.to_rust_string().ok_or_else(|| ParseError {
        offset: 0,
        kind: ParseErrorKind::InvalidText {
            bytes: text.to_modified_utf8().into_boxed_slice(),
        },
    })
}

fn class_name(pool: &ConstantPool, index: u16) -> ParseResult<String> {
    let text = pool
        .class_name(index)
        .ok_or_else(|| pool_error(ParseErrorKind::BadConstantPoolIndex(index)))?;
    let name = text_string(text)?;
    validate_class_name(&name)?;
    Ok(name)
}

fn validate_class_name(name: &str) -> ParseResult<()> {
    if name.is_empty() || name.contains(['.', ';', '[', '(']) {
        return Err(pool_error(ParseErrorKind::InvalidName(
            "invalid class name",
        )));
    }
    Ok(())
}

fn validate_member_name(name: &str, method: bool) -> ParseResult<()> {
    if name.is_empty() || name.contains(['.', ';', '[', '/']) {
        return Err(pool_error(ParseErrorKind::InvalidName(
            "invalid member name",
        )));
    }
    if name.starts_with('<') {
        if !method || (name != "<init>" && name != "<clinit>") {
            return Err(pool_error(ParseErrorKind::InvalidName(
                "invalid special method name",
            )));
        }
    }
    Ok(())
}

fn pool_error(kind: ParseErrorKind) -> ParseError {
    ParseError::new(0, kind)
}
