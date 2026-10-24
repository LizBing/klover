//! Java 8 class-file parser.
//!
//! The pipeline is:
//! `bytes -> decode -> RawClassImage -> validate -> ValidatedClassImage
//!  -> publish -> ClassImage`.
//!
//! `RawClassImage` only checks framing. `validate` checks relationships,
//! descriptors, bytecode, and `Code`. `publish` is a move into the read-only
//! image used by the rest of the crate.
//!
//! The constant pool is a validation detail. Published opcodes carry the
//! entries they need (shared through `Arc` when the same class-file entry is
//! used more than once), and `ClassImage` does not expose `ConstantPool`.

mod byte_reader;
mod bytecode;
mod constant_pool;
mod decode;
mod descriptor;
mod error;
mod flags;
mod model;
mod text;
mod validate;

pub(crate) use bytecode::{Bytecode, Opcode};
#[cfg(test)]
pub(crate) use constant_pool::MemberKind;
pub(crate) use constant_pool::{Constant, LoadableConstant};
pub(crate) use descriptor::{FieldDescriptor, FieldType, MethodDescriptor};
pub(crate) use error::ParseError;
#[cfg(test)]
pub(crate) use error::ParseErrorKind;
pub(crate) use flags::{ClassAccessFlags, MethodAccessFlags};
pub(crate) use model::{ClassImage, CodeData, FieldInfo, MethodInfo};
pub(crate) use text::JavaText;

pub(crate) fn parse_class(bytes: &[u8]) -> Result<ClassImage, ParseError> {
    let raw = decode::decode(bytes)?;
    let validated = validate::validate(raw)?;
    Ok(model::publish(validated))
}

#[cfg(test)]
mod tests;
