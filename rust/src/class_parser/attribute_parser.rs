//! Attribute decoding and local structural validation. This is not a verifier.
use std::collections::HashSet;
use std::sync::Arc;

use super::attributes::*;
use super::byte_reader::ByteReader;
use super::bytecode::{Bytecode, Opcode};
use super::constant_pool::{Constant, ConstantPool, LoadableConstant};
use super::decode::RawAttribute;
use super::descriptor::{FieldDescriptor, FieldType, MethodDescriptor};
use super::entries::{self, ResolvedPool};
use super::error::{ParseError, ParseErrorKind, ParseResult};
use super::flags::MethodAccessFlags;
use super::text::JavaText;

fn invalid(message: &'static str) -> ParseError {
    ParseError::new(0, ParseErrorKind::InvalidAttribute(message))
}
fn text(pool: &ConstantPool, index: u16) -> ParseResult<JavaText> {
    pool.utf8(index)
        .cloned()
        .ok_or_else(|| ParseError::new(0, ParseErrorKind::BadConstantPoolIndex(index)))
}
fn string(pool: &ConstantPool, index: u16) -> ParseResult<String> {
    let value = text(pool, index)?;
    value
        .to_rust_string()
        .ok_or_else(|| invalid("attribute descriptor is not Unicode scalar text"))
}
fn optional_text(pool: &ConstantPool, index: u16) -> ParseResult<Option<JavaText>> {
    if index == 0 {
        Ok(None)
    } else {
        text(pool, index).map(Some)
    }
}
fn finish(reader: &ByteReader<'_>) -> ParseResult<()> {
    if reader.is_empty() {
        Ok(())
    } else {
        Err(invalid("trailing bytes in attribute"))
    }
}
fn unique(seen: &mut HashSet<String>, name: &str) -> ParseResult<()> {
    if seen.insert(name.to_owned()) {
        Ok(())
    } else {
        Err(invalid("duplicate singleton attribute"))
    }
}
fn opaque(attribute: &RawAttribute<'_>, pool: &ConstantPool) -> ParseResult<OpaqueAttribute> {
    Ok(OpaqueAttribute {
        name: text(pool, attribute.name_index)?,
        body: attribute.body.into(),
    })
}
fn name(attribute: &RawAttribute<'_>, pool: &ConstantPool) -> ParseResult<Option<String>> {
    Ok(text(pool, attribute.name_index)?.to_rust_string())
}
fn list<T>(
    reader: &mut ByteReader<'_>,
    mut read: impl FnMut(&mut ByteReader<'_>) -> ParseResult<T>,
) -> ParseResult<Vec<T>> {
    let count = reader.read_u16()?;
    (0..count).map(|_| read(reader)).collect()
}

#[derive(Clone, Copy)]
enum Location {
    Class,
    Field,
    Method,
    Code,
}

/// Shared declaration attributes; returns false for an attribute outside this set.
fn common(
    name: &str,
    reader: &mut ByteReader<'_>,
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
    location: Location,
    annotations: &mut AnnotationAttributes,
    signature: &mut Option<JavaText>,
    synthetic: &mut bool,
    deprecated: &mut bool,
) -> ParseResult<bool> {
    match name {
        "Signature" => *signature = Some(text(pool, reader.read_u16()?)?),
        "Synthetic" => *synthetic = true,
        "Deprecated" => *deprecated = true,
        "RuntimeVisibleAnnotations" => {
            annotations.visible = Some(list(reader, |r| annotation(r, pool, resolved, 0))?)
        }
        "RuntimeInvisibleAnnotations" => {
            annotations.invisible = Some(list(reader, |r| annotation(r, pool, resolved, 0))?)
        }
        "RuntimeVisibleTypeAnnotations" => {
            annotations.visible_types = Some(type_annotations(reader, pool, resolved, location)?)
        }
        "RuntimeInvisibleTypeAnnotations" => {
            annotations.invisible_types = Some(type_annotations(reader, pool, resolved, location)?)
        }
        _ => return Ok(false),
    }
    Ok(true)
}

pub(super) fn class_attributes(
    raw: &[RawAttribute<'_>],
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
) -> ParseResult<ClassAttributes> {
    let mut attrs = ClassAttributes::default();
    let mut seen = HashSet::new();
    for attribute in raw {
        let attribute_name = name(attribute, pool)?;
        let n = attribute_name.as_deref().unwrap_or("");
        let mut r = ByteReader::new(attribute.body);
        let known = match n {
            "BootstrapMethods" => {
                let methods = list(&mut r, |r| {
                    let method = entries::handle(pool, r.read_u16()?, 0, resolved)?;
                    if !matches!(method.kind, 6 | 8) {
                        return Err(invalid(
                            "bootstrap handle must be invokeStatic or newInvokeSpecial",
                        ));
                    }
                    let arguments = list(r, |r| entries::constant(pool, r.read_u16()?, resolved))?;
                    Ok(Arc::new(BootstrapMethod { method, arguments }))
                })?;
                attrs.bootstrap_methods = Some(methods);
                true
            }
            "InnerClasses" => {
                attrs.inner_classes = Some(list(&mut r, |r| {
                    let inner = entries::class_ref(pool, r.read_u16()?, 0, resolved)?;
                    let outer_index = r.read_u16()?;
                    let outer = if outer_index == 0 {
                        None
                    } else {
                        Some(entries::class_ref(pool, outer_index, 0, resolved)?)
                    };
                    let inner_name = optional_text(pool, r.read_u16()?)?;
                    let access_flags = r.read_u16()?;
                    Ok(InnerClass {
                        inner,
                        outer,
                        inner_name,
                        access_flags,
                    })
                })?);
                true
            }
            "EnclosingMethod" => {
                let class = entries::class_ref(pool, r.read_u16()?, 0, resolved)?;
                let index = r.read_u16()?;
                let method = if index == 0 {
                    None
                } else {
                    let (name, descriptor) = pool
                        .name_and_type(index)
                        .ok_or_else(|| invalid("enclosing method must reference NameAndType"))?;
                    let descriptor = descriptor
                        .to_rust_string()
                        .ok_or_else(|| invalid("invalid enclosing method descriptor"))?;
                    Some(EnclosingMethodName {
                        name: name.clone(),
                        descriptor: MethodDescriptor::parse(&descriptor)?,
                    })
                };
                attrs.enclosing_method = Some(EnclosingMethod { class, method });
                true
            }
            "SourceFile" => {
                attrs.source_file = Some(text(pool, r.read_u16()?)?);
                true
            }
            "SourceDebugExtension" => {
                attrs.source_debug_extension = Some(r.read_bytes(r.remaining())?.into());
                true
            }
            _ => common(
                n,
                &mut r,
                pool,
                resolved,
                Location::Class,
                &mut attrs.annotations,
                &mut attrs.signature,
                &mut attrs.synthetic,
                &mut attrs.deprecated,
            )?,
        };
        if known {
            unique(&mut seen, n)?;
            finish(&r)?;
        } else {
            attrs.unknown.push(opaque(attribute, pool)?);
        }
    }
    resolved.bootstraps = attrs.bootstrap_methods.clone().unwrap_or_default();
    // Validate even unused InvokeDynamic entries against the retained table.
    for (_, entry) in pool.iter() {
        if let Constant::InvokeDynamic { bootstrap, .. } = entry {
            if *bootstrap as usize >= resolved.bootstraps.len() {
                return Err(invalid("invalid bootstrap method index"));
            }
        }
    }
    Ok(attrs)
}

pub(super) fn field_attributes(
    raw: &[RawAttribute<'_>],
    descriptor: &FieldDescriptor,
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
) -> ParseResult<FieldAttributes> {
    let mut attrs = FieldAttributes::default();
    let mut seen = HashSet::new();
    for attribute in raw {
        let attribute_name = name(attribute, pool)?;
        let n = attribute_name.as_deref().unwrap_or("");
        let mut r = ByteReader::new(attribute.body);
        let known = if n == "ConstantValue" {
            let value = entries::constant(pool, r.read_u16()?, resolved)?;
            let valid = descriptor.dimensions() == 0
                && matches!(
                    (descriptor.element(), value.as_ref()),
                    (
                        FieldType::Boolean
                            | FieldType::Byte
                            | FieldType::Char
                            | FieldType::Short
                            | FieldType::Int,
                        LoadableConstant::Integer(_)
                    ) | (FieldType::Long, LoadableConstant::Long(_))
                        | (FieldType::Float, LoadableConstant::Float(_))
                        | (FieldType::Double, LoadableConstant::Double(_))
                )
                || descriptor.raw() == "Ljava/lang/String;"
                    && matches!(value.as_ref(), LoadableConstant::String(_));
            if !valid {
                return Err(invalid("ConstantValue does not match field descriptor"));
            }
            attrs.constant_value = Some(value);
            true
        } else {
            common(
                n,
                &mut r,
                pool,
                resolved,
                Location::Field,
                &mut attrs.annotations,
                &mut attrs.signature,
                &mut attrs.synthetic,
                &mut attrs.deprecated,
            )?
        };
        if known {
            unique(&mut seen, n)?;
            finish(&r)?;
        } else {
            attrs.unknown.push(opaque(attribute, pool)?);
        }
    }
    Ok(attrs)
}

pub(super) fn method_attributes(
    raw: &[RawAttribute<'_>],
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
    descriptor: &MethodDescriptor,
    flags: MethodAccessFlags,
) -> ParseResult<MethodAttributes> {
    let mut initial_locals = Vec::new();
    if !flags.contains(MethodAccessFlags::STATIC) {
        initial_locals.push(1);
    }
    initial_locals.extend(
        descriptor
            .parameters()
            .iter()
            .map(|parameter| parameter.slots() as u8),
    );
    let mut attrs = MethodAttributes::default();
    let mut seen = HashSet::new();
    for attribute in raw {
        let attribute_name = name(attribute, pool)?;
        let n = attribute_name.as_deref().unwrap_or("");
        // Check before parsing another Code to avoid unnecessary allocations.
        if n == "Code" && attrs.code.is_some() {
            return Err(ParseError::new(0, ParseErrorKind::MultipleCodeAttributes));
        }
        let mut r = ByteReader::new(attribute.body);
        let known = match n {
            "Code" => {
                attrs.code = Some(super::validate::parse_code(
                    attribute.body,
                    pool,
                    resolved,
                    &initial_locals,
                )?);
                r.skip(r.remaining())?;
                true
            }
            "Exceptions" => {
                attrs.exceptions = Some(list(&mut r, |r| {
                    entries::class_ref(pool, r.read_u16()?, 0, resolved)
                })?);
                true
            }
            "MethodParameters" => {
                let count = r.read_u8()?;
                let mut parameters = Vec::new();
                for _ in 0..count {
                    let name = optional_text(pool, r.read_u16()?)?;
                    let access_flags = r.read_u16()?;
                    if access_flags & !0x9010 != 0 {
                        return Err(invalid("invalid method parameter flags"));
                    }
                    parameters.push(MethodParameter { name, access_flags });
                }
                attrs.parameters = Some(parameters);
                true
            }
            "RuntimeVisibleParameterAnnotations" | "RuntimeInvisibleParameterAnnotations" => {
                let count = r.read_u8()?;
                let parameters = (0..count)
                    .map(|_| list(&mut r, |r| annotation(r, pool, resolved, 0)))
                    .collect::<ParseResult<Vec<_>>>()?;
                if n == "RuntimeVisibleParameterAnnotations" {
                    attrs.visible_parameter_annotations = Some(parameters);
                } else {
                    attrs.invisible_parameter_annotations = Some(parameters);
                }
                true
            }
            "AnnotationDefault" => {
                attrs.annotation_default = Some(element(&mut r, pool, resolved, 0)?);
                true
            }
            _ => common(
                n,
                &mut r,
                pool,
                resolved,
                Location::Method,
                &mut attrs.annotations,
                &mut attrs.signature,
                &mut attrs.synthetic,
                &mut attrs.deprecated,
            )?,
        };
        if known {
            unique(&mut seen, n)?;
            finish(&r)?;
        } else {
            attrs.unknown.push(opaque(attribute, pool)?);
        }
    }
    Ok(attrs)
}

fn object_descriptor(pool: &ConstantPool, index: u16) -> ParseResult<FieldDescriptor> {
    let descriptor = FieldDescriptor::parse(&string(pool, index)?)?;
    if descriptor.dimensions() != 0 || !matches!(descriptor.element(), FieldType::Object(_)) {
        return Err(invalid("expected object descriptor"));
    }
    Ok(descriptor)
}

// Bound recursive parsing so a malicious attribute cannot exhaust the host stack.
const MAX_ANNOTATION_DEPTH: usize = 128;
fn depth_check(depth: usize) -> ParseResult<()> {
    if depth >= MAX_ANNOTATION_DEPTH {
        Err(invalid("annotation nesting limit exceeded"))
    } else {
        Ok(())
    }
}
fn annotation(
    r: &mut ByteReader<'_>,
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
    depth: usize,
) -> ParseResult<Annotation> {
    depth_check(depth)?;
    let annotation_type = object_descriptor(pool, r.read_u16()?)?;
    let mut names = HashSet::new();
    let elements = list(r, |r| {
        let name = text(pool, r.read_u16()?)?;
        if !names.insert(name.clone()) {
            return Err(invalid("duplicate annotation element name"));
        }
        let value = element(r, pool, resolved, depth + 1)?;
        Ok((name, value))
    })?;
    Ok(Annotation {
        annotation_type,
        elements,
    })
}
fn element(
    r: &mut ByteReader<'_>,
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
    depth: usize,
) -> ParseResult<ElementValue> {
    depth_check(depth)?;
    let tag = r.read_u8()?;
    Ok(match tag {
        b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z' => {
            let value = entries::constant(pool, r.read_u16()?, resolved)?;
            let valid = matches!(
                (tag, value.as_ref()),
                (
                    b'B' | b'C' | b'I' | b'S' | b'Z',
                    LoadableConstant::Integer(_)
                ) | (b'D', LoadableConstant::Double(_))
                    | (b'F', LoadableConstant::Float(_))
                    | (b'J', LoadableConstant::Long(_))
            );
            if !valid {
                return Err(invalid("annotation constant has wrong kind"));
            }
            ElementValue::Constant { tag, value }
        }
        b's' => ElementValue::String(text(pool, r.read_u16()?)?),
        b'e' => ElementValue::Enum {
            enum_type: object_descriptor(pool, r.read_u16()?)?,
            name: text(pool, r.read_u16()?)?,
        },
        b'c' => {
            let descriptor = string(pool, r.read_u16()?)?;
            ElementValue::Class(if descriptor == "V" {
                None
            } else {
                Some(FieldDescriptor::parse(&descriptor)?)
            })
        }
        b'@' => ElementValue::Annotation(Box::new(annotation(r, pool, resolved, depth + 1)?)),
        b'[' => ElementValue::Array(list(r, |r| element(r, pool, resolved, depth + 1))?),
        _ => return Err(invalid("invalid annotation element tag")),
    })
}

fn type_annotations(
    r: &mut ByteReader<'_>,
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
    location: Location,
) -> ParseResult<Vec<TypeAnnotation>> {
    list(r, |r| {
        let tag = r.read_u8()?;
        let allowed = match location {
            Location::Class => matches!(tag, 0x00 | 0x10 | 0x11),
            Location::Field => tag == 0x13,
            Location::Method => matches!(tag, 0x01 | 0x12 | 0x14..=0x17),
            Location::Code => matches!(tag, 0x40..=0x4b),
        };
        if !allowed {
            return Err(invalid("invalid type annotation target for location"));
        }
        let target = match tag {
            0x00 => TypeAnnotationTarget::ClassTypeParameter(r.read_u8()?),
            0x01 => TypeAnnotationTarget::MethodTypeParameter(r.read_u8()?),
            0x10 => TypeAnnotationTarget::Supertype(r.read_u16()?),
            0x11 => TypeAnnotationTarget::ClassTypeParameterBound {
                parameter: r.read_u8()?,
                bound: r.read_u8()?,
            },
            0x12 => TypeAnnotationTarget::MethodTypeParameterBound {
                parameter: r.read_u8()?,
                bound: r.read_u8()?,
            },
            0x13 => TypeAnnotationTarget::Field,
            0x14 => TypeAnnotationTarget::MethodReturn,
            0x15 => TypeAnnotationTarget::MethodReceiver,
            0x16 => TypeAnnotationTarget::MethodParameter(r.read_u8()?),
            0x17 => TypeAnnotationTarget::Throws(r.read_u16()?),
            0x40 | 0x41 => {
                let ranges = list(r, read_range)?;
                if tag == 0x40 {
                    TypeAnnotationTarget::LocalVariable(ranges)
                } else {
                    TypeAnnotationTarget::ResourceVariable(ranges)
                }
            }
            0x42 => TypeAnnotationTarget::Catch(r.read_u16()?),
            0x43..=0x46 => TypeAnnotationTarget::Offset {
                target_type: tag,
                offset: r.read_u16()?,
            },
            0x47..=0x4b => TypeAnnotationTarget::TypeArgument {
                target_type: tag,
                offset: r.read_u16()?,
                argument: r.read_u8()?,
            },
            _ => unreachable!(),
        };
        let count = r.read_u8()?;
        let mut path = Vec::new();
        for _ in 0..count {
            let kind = r.read_u8()?;
            let argument = r.read_u8()?;
            path.push(match (kind, argument) {
                (0, 0) => TypePathStep::ArrayElement,
                (1, 0) => TypePathStep::NestedType,
                (2, 0) => TypePathStep::WildcardBound,
                (3, index) => TypePathStep::TypeArgument(index),
                _ => return Err(invalid("invalid type annotation path")),
            });
        }
        Ok(TypeAnnotation {
            target,
            path,
            annotation: annotation(r, pool, resolved, 0)?,
        })
    })
}

fn read_range(r: &mut ByteReader<'_>) -> ParseResult<LocalVariableRange> {
    Ok(LocalVariableRange {
        start_pc: r.read_u16()?,
        length: r.read_u16()?,
        index: r.read_u16()?,
    })
}

pub(super) struct CodeContext<'a> {
    pub(super) bytecode: &'a Bytecode,
    pub(super) code_len: usize,
    pub(super) max_locals: u16,
    pub(super) max_stack: u16,
    pub(super) exception_count: usize,
    pub(super) initial_locals: &'a [u8],
}
impl CodeContext<'_> {
    fn boundary(&self, pc: u16) -> ParseResult<()> {
        if self.bytecode.get_opcode_index(pc as u32).is_some() {
            Ok(())
        } else {
            Err(invalid("attribute offset is not an instruction boundary"))
        }
    }
    fn range(&self, range: &LocalVariableRange, width: u16) -> ParseResult<()> {
        self.boundary(range.start_pc)?;
        let end = usize::from(range.start_pc) + usize::from(range.length);
        if end > self.code_len
            || (end != self.code_len && self.bytecode.get_opcode_index(end as u32).is_none())
        {
            return Err(invalid("invalid local variable range"));
        }
        if u32::from(range.index) + u32::from(width) > u32::from(self.max_locals) {
            return Err(invalid("local variable index exceeds max_locals"));
        }
        Ok(())
    }
}

pub(super) fn code_attributes(
    raw: &[RawAttribute<'_>],
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
    context: &CodeContext<'_>,
) -> ParseResult<CodeAttributes> {
    let mut attrs = CodeAttributes::default();
    let mut seen = HashSet::new();
    for attribute in raw {
        let attribute_name = name(attribute, pool)?;
        let n = attribute_name.as_deref().unwrap_or("");
        let mut r = ByteReader::new(attribute.body);
        let known = match n {
            "StackMapTable" => {
                attrs.stack_map_table = Some(stack_map(&mut r, pool, resolved, context)?);
                true
            }
            "LineNumberTable" => {
                attrs.line_number_tables.push(list(&mut r, |r| {
                    let start_pc = r.read_u16()?;
                    let line_number = r.read_u16()?;
                    context.boundary(start_pc)?;
                    Ok(LineNumber {
                        start_pc,
                        line_number,
                    })
                })?);
                true
            }
            "LocalVariableTable" => {
                attrs.local_variable_tables.push(list(&mut r, |r| {
                    let start_pc = r.read_u16()?;
                    let length = r.read_u16()?;
                    let name = text(pool, r.read_u16()?)?;
                    let descriptor = FieldDescriptor::parse(&string(pool, r.read_u16()?)?)?;
                    let range = LocalVariableRange {
                        start_pc,
                        length,
                        index: r.read_u16()?,
                    };
                    context.range(&range, descriptor.slots())?;
                    Ok(LocalVariable {
                        range,
                        name,
                        descriptor,
                    })
                })?);
                true
            }
            "LocalVariableTypeTable" => {
                attrs.local_variable_type_tables.push(list(&mut r, |r| {
                    let start_pc = r.read_u16()?;
                    let length = r.read_u16()?;
                    let name = text(pool, r.read_u16()?)?;
                    let signature = text(pool, r.read_u16()?)?;
                    let range = LocalVariableRange {
                        start_pc,
                        length,
                        index: r.read_u16()?,
                    };
                    context.range(&range, 1)?;
                    Ok(LocalVariableType {
                        range,
                        name,
                        signature,
                    })
                })?);
                true
            }
            "RuntimeVisibleTypeAnnotations" | "RuntimeInvisibleTypeAnnotations" => {
                let annotations = type_annotations(&mut r, pool, resolved, Location::Code)?;
                for annotation in &annotations {
                    validate_code_target(&annotation.target, context)?;
                }
                if n == "RuntimeVisibleTypeAnnotations" {
                    attrs.visible_type_annotations = Some(annotations);
                } else {
                    attrs.invisible_type_annotations = Some(annotations);
                }
                true
            }
            _ => false,
        };
        if known {
            if !matches!(
                n,
                "LineNumberTable" | "LocalVariableTable" | "LocalVariableTypeTable"
            ) {
                unique(&mut seen, n)?;
            }
            finish(&r)?;
        } else {
            attrs.unknown.push(opaque(attribute, pool)?);
        }
    }
    Ok(attrs)
}

fn validate_code_target(
    target: &TypeAnnotationTarget,
    context: &CodeContext<'_>,
) -> ParseResult<()> {
    match target {
        TypeAnnotationTarget::LocalVariable(ranges)
        | TypeAnnotationTarget::ResourceVariable(ranges) => {
            for range in ranges {
                context.range(range, 1)?;
            }
        }
        TypeAnnotationTarget::Catch(index) => {
            if *index as usize >= context.exception_count {
                return Err(invalid("invalid type annotation exception index"));
            }
        }
        TypeAnnotationTarget::Offset { offset, .. }
        | TypeAnnotationTarget::TypeArgument { offset, .. } => context.boundary(*offset)?,
        _ => unreachable!(),
    }
    Ok(())
}

fn verification_type(
    r: &mut ByteReader<'_>,
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
    context: &CodeContext<'_>,
) -> ParseResult<VerificationType> {
    Ok(match r.read_u8()? {
        0 => VerificationType::Top,
        1 => VerificationType::Integer,
        2 => VerificationType::Float,
        3 => VerificationType::Double,
        4 => VerificationType::Long,
        5 => VerificationType::Null,
        6 => VerificationType::UninitializedThis,
        7 => VerificationType::Object(entries::class_ref(pool, r.read_u16()?, 0, resolved)?),
        8 => {
            let offset = r.read_u16()?;
            let index = context
                .bytecode
                .get_opcode_index(offset as u32)
                .ok_or_else(|| invalid("invalid uninitialized offset"))?;
            if !matches!(context.bytecode.opcodes[index].1, Opcode::New(_)) {
                return Err(invalid("uninitialized offset must identify new"));
            }
            VerificationType::Uninitialized(offset)
        }
        _ => return Err(invalid("invalid verification type tag")),
    })
}

fn stack_map(
    r: &mut ByteReader<'_>,
    pool: &ConstantPool,
    resolved: &mut ResolvedPool,
    context: &CodeContext<'_>,
) -> ParseResult<Vec<StackMapFrame>> {
    let count = r.read_u16()?;
    let mut frames = Vec::new();
    let mut previous: Option<u16> = None;
    let mut local_widths = context.initial_locals.to_vec();
    for _ in 0..count {
        let tag = r.read_u8()?;
        let delta = match tag {
            0..=63 => tag as u16,
            64..=127 => (tag - 64) as u16,
            247..=255 => r.read_u16()?,
            _ => return Err(invalid("reserved stack map frame tag")),
        };
        let kind = match tag {
            0..=63 | 251 => StackMapFrameKind::Same,
            64..=127 | 247 => StackMapFrameKind::SameLocalsOneStack(verification_type(
                r, pool, resolved, context,
            )?),
            248..=250 => StackMapFrameKind::Chop(251 - tag),
            252..=254 => StackMapFrameKind::Append(
                (0..tag - 251)
                    .map(|_| verification_type(r, pool, resolved, context))
                    .collect::<ParseResult<_>>()?,
            ),
            255 => StackMapFrameKind::Full {
                locals: list(r, |r| verification_type(r, pool, resolved, context))?,
                stack: list(r, |r| verification_type(r, pool, resolved, context))?,
            },
            _ => unreachable!(),
        };
        let offset = match previous {
            None => delta as u32,
            Some(pc) => u32::from(pc) + u32::from(delta) + 1,
        };
        let offset = u16::try_from(offset).map_err(|_| invalid("stack map offset overflow"))?;
        context.boundary(offset)?;
        match &kind {
            StackMapFrameKind::Chop(count) => {
                let length = local_widths
                    .len()
                    .checked_sub(*count as usize)
                    .ok_or_else(|| invalid("stack map chops more locals than available"))?;
                local_widths.truncate(length);
            }
            StackMapFrameKind::Append(locals) => {
                local_widths.extend(locals.iter().map(verification_width))
            }
            StackMapFrameKind::Full { locals, stack } => {
                local_widths = locals.iter().map(verification_width).collect();
                if slots(stack) > context.max_stack as usize {
                    return Err(invalid("stack map exceeds max_stack"));
                }
            }
            _ => {}
        }
        if local_widths
            .iter()
            .map(|width| *width as usize)
            .sum::<usize>()
            > context.max_locals as usize
        {
            return Err(invalid("stack map exceeds max_locals"));
        }
        if let StackMapFrameKind::SameLocalsOneStack(item) = &kind {
            if slots(std::slice::from_ref(item)) > context.max_stack as usize {
                return Err(invalid("stack map exceeds max_stack"));
            }
        }
        frames.push(StackMapFrame { offset, kind });
        previous = Some(offset);
    }
    Ok(frames)
}
fn verification_width(ty: &VerificationType) -> u8 {
    if matches!(ty, VerificationType::Double | VerificationType::Long) {
        2
    } else {
        1
    }
}
fn slots(types: &[VerificationType]) -> usize {
    types.iter().map(|ty| verification_width(ty) as usize).sum()
}
