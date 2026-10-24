//! Published class image and its private validation-time data.

use super::bytecode::Bytecode;
use super::constant_pool::{Constant, ConstantPool};
use super::descriptor::{FieldDescriptor, MethodDescriptor};
use super::flags::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

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
    pub(super) this_class: String,
    pub(super) super_class: Option<String>,
    pub(super) interfaces: Vec<String>,
    pub(super) fields: Vec<FieldInfo>,
    pub(super) methods: Vec<MethodInfo>,
    pub(super) constant_pool: ConstantPool,
}

#[derive(Debug)]
pub(crate) struct FieldInfo {
    pub(crate) access_flags: FieldAccessFlags,
    pub(crate) name: String,
    pub(crate) descriptor: FieldDescriptor,
}

#[derive(Debug)]
pub(crate) struct MethodInfo {
    pub(crate) access_flags: MethodAccessFlags,
    pub(crate) name: String,
    pub(crate) descriptor: MethodDescriptor,
    pub(crate) code: Option<CodeData>,
}

#[derive(Debug)]
pub(crate) struct CodeData {
    pub(crate) max_stack: u16,
    pub(crate) max_locals: u16,
    pub(crate) bytecode: Bytecode,
    pub(crate) exception_table: Vec<ExceptionHandler>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExceptionHandler {
    pub(crate) start_pc: u16,
    pub(crate) end_pc: u16,
    pub(crate) handler_pc: u16,
    /// Zero means catch-all.
    pub(crate) catch_type: u16,
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

    pub(crate) fn this_class_name(&self) -> &str {
        &self.data.this_class
    }

    pub(crate) fn super_class_name(&self) -> Option<&str> {
        self.data.super_class.as_deref()
    }

    pub(crate) fn interfaces(&self) -> &[String] {
        &self.data.interfaces
    }

    pub(crate) fn fields(&self) -> &[FieldInfo] {
        &self.data.fields
    }

    pub(crate) fn methods(&self) -> &[MethodInfo] {
        &self.data.methods
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
