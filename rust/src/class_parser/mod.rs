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
//! Instructions and typed attributes share canonical symbolic entries through
//! `Arc`, keyed internally by original CP index. No runtime resolution occurs.
//! The original pool remains available through checked read-only lookup for
//! consumers of opaque attributes. See README.md for the API and its limits.

mod attribute_parser;
pub(crate) mod attributes;
mod byte_reader;
mod bytecode;
mod constant_pool;
mod decode;
mod descriptor;
mod entries;
mod error;
mod flags;
mod model;
mod text;
mod validate;

pub(crate) use bytecode::{Bytecode, Opcode};
#[allow(unused_imports)]
pub(crate) use constant_pool::{
    ClassMemberRef, ClassRef, Constant, InvokeDynamicConstant, LoadableConstant, MemberKind,
    MethodHandleConstant,
};
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

#[cfg(test)]
mod attribute_tests;
