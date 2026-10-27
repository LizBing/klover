//! Canonical symbolic entries, shared by instructions and attributes.
//! Identity is per original CP index, never deduplicated by contents.
use super::attributes::BootstrapMethod;
use super::constant_pool::{
    ClassMemberRef, ClassRef, Constant, ConstantPool, InvokeDynamicConstant, LoadableConstant,
    MemberKind, MethodHandleConstant,
};
use super::error::{ParseError, ParseErrorKind, ParseResult};
use std::collections::HashMap;
use std::sync::Arc;
/// Per-class symbolic-entry builder, shared by all attributes and methods.
/// The cache is dropped after validation; published Arcs keep allocations alive.
/// "Resolved" here means decoded CP relationships, not JVM runtime resolution.
#[derive(Default)]
pub(super) struct ResolvedPool {
    classes: HashMap<u16, Arc<ClassRef>>,
    handles: HashMap<u16, Arc<MethodHandleConstant>>,
    pub(super) bootstraps: Vec<Arc<BootstrapMethod>>,
    members: HashMap<u16, Arc<ClassMemberRef>>,
    loadables: HashMap<u16, Arc<LoadableConstant>>,
    dynamics: HashMap<u16, Arc<InvokeDynamicConstant>>,
}

fn text_string(text: &super::text::JavaText, pc: u32) -> ParseResult<String> {
    text.to_rust_string().ok_or_else(|| ParseError {
        offset: pc as usize,
        kind: ParseErrorKind::InvalidText {
            bytes: text.to_modified_utf8().into_boxed_slice(),
        },
    })
}

fn class_name(pool: &ConstantPool, index: u16, pc: u32) -> ParseResult<String> {
    let text = pool.class_name(index).ok_or_else(|| {
        if pool.entry(index).is_none() {
            invalid(pc, ParseErrorKind::BadConstantPoolIndex(index))
        } else {
            invalid(
                pc,
                ParseErrorKind::BadConstantPoolEntry("expected class entry"),
            )
        }
    })?;
    text_string(text, pc)
}

pub(super) fn member(
    pool: &ConstantPool,
    index: u16,
    expected: MemberKind,
    pc: u32,
    resolved: &mut ResolvedPool,
) -> ParseResult<Arc<ClassMemberRef>> {
    if let Some(cached) = resolved.members.get(&index) {
        if cached.kind == expected {
            return Ok(cached.clone());
        }
        return Err(invalid(
            pc,
            ParseErrorKind::BadConstantPoolEntry("wrong member reference kind"),
        ));
    }
    let (owner, name_and_type, kind) = match (expected, pool.entry(index)) {
        (
            MemberKind::Field,
            Some(Constant::FieldRef {
                owner,
                name_and_type,
            }),
        ) => (*owner, *name_and_type, MemberKind::Field),
        (
            MemberKind::Method,
            Some(Constant::MethodRef {
                owner,
                name_and_type,
            }),
        ) => (*owner, *name_and_type, MemberKind::Method),
        (
            MemberKind::InterfaceMethod,
            Some(Constant::InterfaceMethodRef {
                owner,
                name_and_type,
            }),
        ) => (*owner, *name_and_type, MemberKind::InterfaceMethod),
        (_, None) => return Err(invalid(pc, ParseErrorKind::BadConstantPoolIndex(index))),
        _ => {
            return Err(invalid(
                pc,
                ParseErrorKind::BadConstantPoolEntry("wrong member reference kind"),
            ));
        }
    };
    let value = Arc::new(resolve_member(
        pool,
        owner,
        name_and_type,
        kind,
        pc,
        resolved,
    )?);
    resolved.members.insert(index, value.clone());
    Ok(value)
}

pub(super) fn method_member(
    pool: &ConstantPool,
    index: u16,
    pc: u32,
    resolved: &mut ResolvedPool,
) -> ParseResult<Arc<ClassMemberRef>> {
    if let Some(cached) = resolved.members.get(&index) {
        if matches!(
            cached.kind,
            MemberKind::Method | MemberKind::InterfaceMethod
        ) {
            return Ok(cached.clone());
        }
        return Err(invalid(
            pc,
            ParseErrorKind::BadConstantPoolEntry("expected method reference"),
        ));
    }
    let value = match pool.entry(index) {
        Some(Constant::MethodRef {
            owner,
            name_and_type,
        }) => resolve_member(
            pool,
            *owner,
            *name_and_type,
            MemberKind::Method,
            pc,
            resolved,
        ),
        Some(Constant::InterfaceMethodRef {
            owner,
            name_and_type,
        }) => resolve_member(
            pool,
            *owner,
            *name_and_type,
            MemberKind::InterfaceMethod,
            pc,
            resolved,
        ),
        Some(_) => Err(invalid(
            pc,
            ParseErrorKind::BadConstantPoolEntry("expected method reference"),
        )),
        None => Err(invalid(pc, ParseErrorKind::BadConstantPoolIndex(index))),
    }?;
    let value = Arc::new(value);
    resolved.members.insert(index, value.clone());
    Ok(value)
}

fn resolve_member(
    pool: &ConstantPool,
    owner: u16,
    name_and_type: u16,
    kind: MemberKind,
    pc: u32,
    resolved: &mut ResolvedPool,
) -> ParseResult<ClassMemberRef> {
    let owner = class_ref(pool, owner, pc, resolved)?;
    let (name, descriptor) = pool
        .name_and_type(name_and_type)
        .ok_or_else(|| invalid(pc, ParseErrorKind::BadConstantPoolIndex(name_and_type)))?;
    Ok(ClassMemberRef {
        kind,
        owner,
        name: text_string(name, pc)?,
        descriptor: text_string(descriptor, pc)?,
    })
}

pub(super) fn loadable(
    pool: &ConstantPool,
    index: u16,
    two_slots: bool,
    pc: u32,
    resolved: &mut ResolvedPool,
) -> ParseResult<Arc<LoadableConstant>> {
    if let Some(cached) = resolved.loadables.get(&index) {
        let valid = matches!(
            (&**cached, two_slots),
            (
                LoadableConstant::Long(_) | LoadableConstant::Double(_),
                true
            ) | (
                LoadableConstant::Integer(_)
                    | LoadableConstant::Float(_)
                    | LoadableConstant::String(_)
                    | LoadableConstant::Class(_)
                    | LoadableConstant::MethodType(_)
                    | LoadableConstant::MethodHandle(_),
                false
            )
        );
        if valid {
            return Ok(cached.clone());
        }
        return Err(invalid(
            pc,
            ParseErrorKind::BadConstantPoolEntry("invalid ldc entry kind"),
        ));
    }
    let entry = pool
        .entry(index)
        .ok_or_else(|| invalid(pc, ParseErrorKind::BadConstantPoolIndex(index)))?;
    let value = match entry {
        Constant::Integer(value) if !two_slots => LoadableConstant::Integer(*value),
        Constant::Float(bits) if !two_slots => LoadableConstant::Float(f32::from_bits(*bits)),
        Constant::Long(value) if two_slots => LoadableConstant::Long(*value),
        Constant::Double(bits) if two_slots => LoadableConstant::Double(f64::from_bits(*bits)),
        Constant::String(text) if !two_slots => LoadableConstant::String(
            pool.utf8(*text)
                .ok_or_else(|| invalid(pc, ParseErrorKind::BadConstantPoolIndex(*text)))?
                .clone(),
        ),
        Constant::Class(_) if !two_slots => {
            LoadableConstant::Class(class_ref(pool, index, pc, resolved)?)
        }
        Constant::MethodType(descriptor) if !two_slots => {
            LoadableConstant::MethodType(text_string(
                pool.utf8(*descriptor).ok_or_else(|| {
                    invalid(pc, ParseErrorKind::BadConstantPoolIndex(*descriptor))
                })?,
                pc,
            )?)
        }
        Constant::MethodHandle { .. } if !two_slots => {
            LoadableConstant::MethodHandle(handle(pool, index, pc, resolved)?)
        }
        _ => {
            return Err(invalid(
                pc,
                ParseErrorKind::BadConstantPoolEntry("invalid ldc entry kind"),
            ));
        }
    };
    let value = Arc::new(value);
    resolved.loadables.insert(index, value.clone());
    Ok(value)
}

fn method_handle(
    pool: &ConstantPool,
    kind: u8,
    reference: u16,
    pc: u32,
    resolved: &mut ResolvedPool,
) -> ParseResult<MethodHandleConstant> {
    let target = match kind {
        1..=4 => member(pool, reference, MemberKind::Field, pc, resolved)?,
        5 | 8 => member(pool, reference, MemberKind::Method, pc, resolved)?,
        6 | 7 => method_member(pool, reference, pc, resolved)?,
        9 => member(pool, reference, MemberKind::InterfaceMethod, pc, resolved)?,
        _ => {
            return Err(invalid(
                pc,
                ParseErrorKind::BadConstantPoolEntry("invalid method-handle kind"),
            ));
        }
    };
    Ok(MethodHandleConstant { kind, target })
}

pub(super) fn invoke_dynamic(
    pool: &ConstantPool,
    index: u16,
    pc: u32,
    resolved: &mut ResolvedPool,
) -> ParseResult<Arc<InvokeDynamicConstant>> {
    if let Some(cached) = resolved.dynamics.get(&index) {
        return Ok(cached.clone());
    }
    let (bootstrap, name_and_type) = match pool.entry(index) {
        Some(Constant::InvokeDynamic {
            bootstrap,
            name_and_type,
        }) => (*bootstrap, *name_and_type),
        Some(_) => {
            return Err(invalid(
                pc,
                ParseErrorKind::BadConstantPoolEntry("expected invokedynamic entry"),
            ));
        }
        None => return Err(invalid(pc, ParseErrorKind::BadConstantPoolIndex(index))),
    };
    let (name, descriptor) = pool
        .name_and_type(name_and_type)
        .ok_or_else(|| invalid(pc, ParseErrorKind::BadConstantPoolIndex(name_and_type)))?;
    let value = Arc::new(InvokeDynamicConstant {
        bootstrap: resolved
            .bootstraps
            .get(bootstrap as usize)
            .cloned()
            .ok_or_else(|| {
                invalid(
                    pc,
                    ParseErrorKind::InvalidAttribute("invalid bootstrap method index"),
                )
            })?,
        name: text_string(name, pc)?,
        descriptor: text_string(descriptor, pc)?,
    });
    resolved.dynamics.insert(index, value.clone());
    Ok(value)
}

/// Returns the canonical allocation for this CONSTANT_Class entry.
pub(super) fn class_ref(
    pool: &ConstantPool,
    index: u16,
    pc: u32,
    resolved: &mut ResolvedPool,
) -> ParseResult<Arc<ClassRef>> {
    if let Some(value) = resolved.classes.get(&index) {
        return Ok(value.clone());
    }
    let value = Arc::new(ClassRef {
        name: class_name(pool, index, pc)?,
    });
    resolved.classes.insert(index, value.clone());
    Ok(value)
}

pub(super) fn handle(
    pool: &ConstantPool,
    index: u16,
    pc: u32,
    resolved: &mut ResolvedPool,
) -> ParseResult<Arc<MethodHandleConstant>> {
    if let Some(value) = resolved.handles.get(&index) {
        return Ok(value.clone());
    }
    let Some(Constant::MethodHandle { kind, reference }) = pool.entry(index) else {
        return Err(invalid(
            pc,
            ParseErrorKind::BadConstantPoolEntry("expected method handle"),
        ));
    };
    let value = Arc::new(method_handle(pool, *kind, *reference, pc, resolved)?);
    resolved.handles.insert(index, value.clone());
    Ok(value)
}

/// Attribute operands allow both category-one and category-two constants.
pub(super) fn constant(
    pool: &ConstantPool,
    index: u16,
    resolved: &mut ResolvedPool,
) -> ParseResult<Arc<LoadableConstant>> {
    let wide = matches!(
        pool.entry(index),
        Some(Constant::Long(_) | Constant::Double(_))
    );
    loadable(pool, index, wide, 0, resolved)
}

fn invalid(pc: u32, kind: ParseErrorKind) -> ParseError {
    ParseError::new(pc as usize, kind)
}
