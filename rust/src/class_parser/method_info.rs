use crate::class_loader::symbol_handle::SymbolHandle;
use crate::class_parser::class_file::{read_acc_flags, read_attrs, resolve_symbol};
use crate::class_parser::field_type::FieldType;
use crate::class_parser::parse_error::ParseError;
use crate::class_parser::{acc_flags::AccFlags, attr_info::AttrInfo};

use super::{class_reader::ClassReader, parse_error::ParseResult};

/// Parsed method metadata — names resolved to `SymbolHandle`, attributes parsed.
///
/// This is the parsed, ready-to-use representation within a [`ClassFile`].
/// The runtime counterpart (vtable index, entry point, etc.) will be a
/// separate `Method` type allocated in metaspace alongside `Klass`.
pub struct MethodInfo {
    pub acc_flags: AccFlags,
    pub name: SymbolHandle,
    pub para_desc: Vec<FieldType>,
    pub ret_desc: Option<FieldType>,
    pub attrs: Vec<AttrInfo>,
}

fn resolve_method_desc(raw: String) -> ParseResult<(Vec<FieldType>, Option<FieldType>)> {
    let close_paren = raw
        .find(')')
        .ok_or(ParseError::InvalidDescriptor { raw: raw.clone() })?;

    if !raw.starts_with('(') {
        return Err(ParseError::InvalidDescriptor { raw });
    }

    let params_raw = &raw[1..close_paren];
    let ret_raw = &raw[close_paren + 1..];

    // Parse parameter types: split concatenated field descriptors
    let mut para_desc = Vec::new();
    let mut chars = params_raw.chars().peekable();
    loop {
        let mut single = String::new();
        // Read one field type descriptor
        match chars.next() {
            None => break,
            Some('L') => {
                single.push('L');
                // Read until ';'
                for c in chars.by_ref() {
                    single.push(c);
                    if c == ';' {
                        break;
                    }
                }
            }
            Some('[') => {
                single.push('[');
                // Consume all leading '['
                while let Some(&c) = chars.peek() {
                    if c == '[' {
                        single.push('[');
                        chars.next();
                    } else {
                        break;
                    }
                }
                // Now read the element type
                match chars.peek() {
                    Some('L') => {
                        single.push('L');
                        chars.next();
                        for c in chars.by_ref() {
                            single.push(c);
                            if c == ';' {
                                break;
                            }
                        }
                    }
                    Some(_) => {
                        // Primitive element type (single char)
                        single.push(chars.next().unwrap());
                    }
                    None => {
                        return Err(super::parse_error::ParseError::InvalidDescriptor { raw });
                    }
                }
            }
            Some(c) => {
                // Primitive type: single char
                single.push(c);
            }
        }
        para_desc.push(FieldType::resolve(single)?);
    }

    // Parse return type
    let ret_desc = if ret_raw == "V" {
        None
    } else {
        Some(FieldType::resolve(ret_raw.to_string())?)
    };

    Ok((para_desc, ret_desc))
}

impl MethodInfo {
    /// Read and resolve a method from the class file stream.
    pub fn read(
        rd: &mut ClassReader,
        cp: &[super::cp_info::ConstantPoolInfo],
    ) -> ParseResult<Self> {
        let acc_flags = read_acc_flags(rd)?;

        let name_index = rd.read_u16()?;
        let desc_index = rd.read_u16()?;

        let attrs = read_attrs(rd, cp)?;

        let name = resolve_symbol(name_index, cp)?;
        let raw_desc = resolve_symbol(desc_index, cp)?.utf8().clone();
        let (para_desc, ret_desc) = resolve_method_desc(raw_desc)?;

        Ok(Self {
            acc_flags,
            name,
            para_desc,
            ret_desc,
            attrs,
        })
    }
}
