//! Class-file constant pool and the self-contained entries carried by
//! `Opcode`.
//!
//! ClassImage provides checked lookup in the original pool for opaque data.
//! Instructions and typed attributes carry canonical symbolic entries.
//! `MethodRef` and `InterfaceMethodRef` remain distinct in both the pool and
//! the resolved member reference.

use std::sync::Arc;

use super::text::JavaText;

#[derive(Debug)]
pub(crate) enum Constant {
    Unusable,
    Utf8(JavaText),
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

/// The class-file reference kind, preserved from the constant-pool tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MemberKind {
    Field,
    Method,
    InterfaceMethod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClassRef {
    pub(crate) name: String,
}

/// A resolved symbolic member reference carried by a field or invoke opcode.
#[derive(Debug, Clone)]
pub(crate) struct ClassMemberRef {
    pub(crate) kind: MemberKind,
    pub(crate) owner: Arc<ClassRef>,
    pub(crate) name: String,
    pub(crate) descriptor: String,
}

#[derive(Debug, Clone)]
pub(crate) struct MethodHandleConstant {
    pub(crate) kind: u8,
    pub(crate) target: Arc<ClassMemberRef>,
}

#[derive(Debug, Clone)]
pub(crate) struct InvokeDynamicConstant {
    pub(crate) bootstrap: Arc<super::attributes::BootstrapMethod>,
    pub(crate) name: String,
    pub(crate) descriptor: String,
}

/// A constant loaded by `ldc`, `ldc_w`, or `ldc2_w`.
#[derive(Debug, Clone)]
pub(crate) enum LoadableConstant {
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    String(JavaText),
    Class(Arc<ClassRef>),
    MethodType(String),
    MethodHandle(Arc<MethodHandleConstant>),
}

#[derive(Debug)]
pub(super) struct ConstantPool {
    entries: Vec<Constant>,
}

impl ConstantPool {
    pub(super) fn new(entries: Vec<Constant>) -> Self {
        Self { entries }
    }

    #[cfg(test)]
    pub(super) fn empty() -> Self {
        Self {
            entries: vec![Constant::Unusable],
        }
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    /// Iterates usable class-file constant-pool entries as `(index, entry)`.
    ///
    /// Index 0 and the unusable second slot of `Long`/`Double` entries are
    /// skipped. `MethodRef` and `InterfaceMethodRef` stay distinct.
    pub(super) fn iter(&self) -> impl Iterator<Item = (u16, &Constant)> {
        self.entries
            .iter()
            .enumerate()
            .skip(1)
            .filter_map(|(index, constant)| {
                if matches!(constant, Constant::Unusable) {
                    None
                } else {
                    Some((index as u16, constant))
                }
            })
    }

    pub(super) fn entry(&self, index: u16) -> Option<&Constant> {
        if index == 0 {
            return None;
        }
        self.entries.get(index as usize)
    }

    pub(super) fn utf8(&self, index: u16) -> Option<&JavaText> {
        match self.entry(index)? {
            Constant::Utf8(text) => Some(text),
            _ => None,
        }
    }

    pub(super) fn class_name(&self, index: u16) -> Option<&JavaText> {
        match self.entry(index)? {
            Constant::Class(name) => self.utf8(*name),
            _ => None,
        }
    }

    pub(super) fn name_and_type(&self, index: u16) -> Option<(&JavaText, &JavaText)> {
        match self.entry(index)? {
            Constant::NameAndType { name, descriptor } => {
                Some((self.utf8(*name)?, self.utf8(*descriptor)?))
            }
            _ => None,
        }
    }
}
