//! Published class image and its private validation-time data.

use super::attributes::{ClassAttributes, CodeAttributes, FieldAttributes, MethodAttributes};
use super::bytecode::Bytecode;
use super::constant_pool::ClassRef;
use super::constant_pool::{Constant, ConstantPool};
use super::descriptor::{FieldDescriptor, MethodDescriptor};
use super::flags::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct ClassImage {
    data: ClassData,
}

pub(super) struct ValidatedClassImage {
    data: ClassData,
}

#[derive(Debug)]
pub(super) struct ClassData {
    pub(super) minor: u16,
    pub(super) major: u16,
    pub(super) access_flags: ClassAccessFlags,
    pub(super) this_class: Arc<ClassRef>,
    pub(super) super_class: Option<Arc<ClassRef>>,
    pub(super) interfaces: Vec<Arc<ClassRef>>,
    pub(super) fields: Vec<FieldInfo>,
    pub(super) methods: Vec<MethodInfo>,
    pub(super) constant_pool: ConstantPool,
    pub(super) attributes: ClassAttributes,
}

#[derive(Debug)]
pub(crate) struct FieldInfo {
    pub(crate) access_flags: FieldAccessFlags,
    pub(crate) name: String,
    pub(crate) descriptor: FieldDescriptor,
    pub(crate) attributes: FieldAttributes,
}

#[derive(Debug)]
pub(crate) struct MethodInfo {
    pub(crate) access_flags: MethodAccessFlags,
    pub(crate) name: String,
    pub(crate) descriptor: MethodDescriptor,
    pub(crate) attributes: MethodAttributes,
}

#[derive(Debug)]
pub(crate) struct CodeData {
    pub(crate) max_stack: u16,
    pub(crate) max_locals: u16,
    pub(crate) bytecode: Bytecode,
    pub(crate) exception_table: Vec<ExceptionHandler>,
    pub(crate) attributes: CodeAttributes,
}

#[derive(Debug, Clone)]
pub(crate) struct ExceptionHandler {
    pub(crate) start_pc: u16,
    pub(crate) end_pc: u16,
    pub(crate) handler_pc: u16,
    /// None means catch-all.
    pub(crate) catch_type: Option<Arc<ClassRef>>,
}

impl ClassImage {
    pub(crate) fn minor_version(&self) -> u16 {
        self.data.minor
    }

    pub(crate) fn major_version(&self) -> u16 {
        self.data.major
    }

    pub(crate) fn access_flags(&self) -> ClassAccessFlags {
        self.data.access_flags
    }

    pub(crate) fn this_class(&self) -> &Arc<ClassRef> {
        &self.data.this_class
    }

    pub(crate) fn super_class(&self) -> Option<&Arc<ClassRef>> {
        self.data.super_class.as_ref()
    }

    // Runtime defense tests deliberately reduce capacity after parsing a valid
    // class, so they do not depend on the parser accepting malformed input.
    #[cfg(test)]
    pub(crate) fn set_max_locals_for_test(&mut self, method: usize, max_locals: u16) {
        self.data.methods[method]
            .attributes
            .code
            .as_mut()
            .unwrap()
            .max_locals = max_locals;
    }

    pub(crate) fn this_class_name(&self) -> &str {
        &self.data.this_class.name
    }

    pub(crate) fn super_class_name(&self) -> Option<&str> {
        self.data
            .super_class
            .as_ref()
            .map(|class| class.name.as_str())
    }

    pub(crate) fn interfaces(&self) -> &[Arc<ClassRef>] {
        &self.data.interfaces
    }

    pub(crate) fn fields(&self) -> &[FieldInfo] {
        &self.data.fields
    }

    pub(crate) fn methods(&self) -> &[MethodInfo] {
        &self.data.methods
    }

    pub(crate) fn attributes(&self) -> &ClassAttributes {
        &self.data.attributes
    }

    /// Checked lookup in the original CP, including for opaque attribute bodies.
    /// Zero, out-of-range indices, and unusable Long/Double slots return None.
    pub(crate) fn constant(&self, index: u16) -> Option<&Constant> {
        self.data
            .constant_pool
            .entry(index)
            .filter(|entry| !matches!(entry, Constant::Unusable))
    }

    /// Iterates the class-file constant-pool entries as `(index, entry)` pairs.
    ///
    /// Index 0 and the unusable second slot of `Long`/`Double` entries are
    /// skipped. This is a read-only view; runtime lowering uses the entries
    /// already carried by opcodes.
    pub(crate) fn constant_pool(&self) -> impl Iterator<Item = (u16, &Constant)> + '_ {
        self.data.constant_pool.iter()
    }
}

impl ValidatedClassImage {
    pub(super) fn new(data: ClassData) -> Self {
        Self { data }
    }
}

pub(super) fn publish(validated: ValidatedClassImage) -> ClassImage {
    ClassImage {
        data: validated.data,
    }
}
