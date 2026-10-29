//! Class-file constant pool and the self-contained entries carried by
//! `Opcode`.
//!
//! The original pool stays internal to the parser.
//! Instructions and typed attributes carry canonical symbolic entries.
//! Method and field references have separate types. `MethodMemRef.kind`
//! preserves the class/interface method distinction from the original CP tag.

use std::sync::Arc;

use super::descriptor::{FieldDescriptor, MethodDescriptor};
use super::text::JavaText;

#[derive(Debug)]
pub(super) enum Constant {
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

/// The original method reference tag, preserved independently of invocation kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MethodMemRefKind {
    Class,
    Interface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClassRef {
    pub(crate) name: String,
}

/// A symbolic CONSTANT_Fieldref entry.
#[derive(Debug, Clone)]
pub(crate) struct FieldMemRef {
    pub(crate) owner: Arc<ClassRef>,
    pub(crate) name: String,
    pub(crate) descriptor: FieldDescriptor,
}

/// A symbolic CONSTANT_Methodref or CONSTANT_InterfaceMethodref entry.
#[derive(Debug, Clone)]
pub(crate) struct MethodMemRef {
    pub(crate) kind: MethodMemRefKind,
    pub(crate) owner: Arc<ClassRef>,
    pub(crate) name: String,
    pub(crate) descriptor: MethodDescriptor,
}

#[derive(Debug, Clone)]
pub(crate) enum MethodHandleTarget {
    Field(Arc<FieldMemRef>),
    Method(Arc<MethodMemRef>),
}

#[derive(Debug, Clone)]
pub(crate) struct MethodHandleConstant {
    pub(crate) kind: u8,
    pub(crate) target: MethodHandleTarget,
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
