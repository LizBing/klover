use std::{mem::size_of, sync::{OnceLock, atomic::AtomicPtr}};

use crate::{
    class_loader::ms_api::MSRef, oops::{
        klass::Klass,
        oop_handle::NObjPtr,
        resolve_error::{ResolveError, ResolveResult},
        symbol_table::{SymbolHandle, SymbolTable},
    }
};

#[derive(Debug)]
pub enum FieldElemType {
    Boolean,
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,

    Class {
        name: SymbolHandle,
        resolved: OnceLock<MSRef<Klass>>,
    },
}

#[derive(Debug)]
pub struct FieldDesc {
    pub dimensions: usize,
    pub elem: FieldElemType,
}

impl FieldDesc {
    pub fn byte_size(&self) -> usize {
        if self.dimensions != 0 {
            return size_of::<NObjPtr>();
        }

        match self.elem {
            FieldElemType::Boolean => size_of::<u8>(),
            FieldElemType::Byte => size_of::<u8>(),
            FieldElemType::Char => size_of::<u16>(),
            FieldElemType::Double => size_of::<f64>(),
            FieldElemType::Float => size_of::<f32>(),
            FieldElemType::Int => size_of::<i32>(),
            FieldElemType::Long => size_of::<i64>(),
            FieldElemType::Short => size_of::<i16>(),
            FieldElemType::Class { .. } => size_of::<NObjPtr>(),
        }
    }
}

impl FieldDesc {
    pub fn from(utf8: &str) -> ResolveResult<Self> {
        let bytes = utf8.as_bytes();
        let mut pos = 0;

        // Parse array dimensions
        let mut dimensions = 0usize;
        while pos < bytes.len() && bytes[pos] == b'[' {
            dimensions += 1;
            pos += 1;
        }

        if pos >= bytes.len() {
            return Err(ResolveError::InvalidDesc { raw: utf8.into() });
        }

        let elem = match bytes[pos] {
            b'B' => FieldElemType::Byte,
            b'C' => FieldElemType::Char,
            b'D' => FieldElemType::Double,
            b'F' => FieldElemType::Float,
            b'I' => FieldElemType::Int,
            b'J' => FieldElemType::Long,
            b'S' => FieldElemType::Short,
            b'Z' => FieldElemType::Boolean,
            b'L' => {
                // Class type: L<classname>;
                let start = pos + 1;
                let end = bytes[start..]
                    .iter()
                    .position(|&b| b == b';')
                    .ok_or_else(|| ResolveError::InvalidDesc { raw: utf8.into() })?;
                let class_name = &utf8[start..start + end];
                FieldElemType::Class {
                    name: SymbolTable::intern(class_name),
                    resolved: OnceLock::new(),
                }
            }
            _ => return Err(ResolveError::InvalidDesc { raw: utf8.into() }),
        };

        Ok(FieldDesc { dimensions, elem })
    }
}

#[derive(Debug)]
pub enum ReturnDesc {
    Void,
    Type(FieldDesc),
}

#[derive(Debug)]
pub struct MethodDesc {
    pub raw: SymbolHandle,
    pub ret_desc: ReturnDesc,
    pub params_desc: Vec<FieldDesc>,
}

impl MethodDesc {
    pub fn from(utf8: &str) -> ResolveResult<Self> {
        let bytes = utf8.as_bytes();

        if bytes.is_empty() || bytes[0] != b'(' {
            return Err(ResolveError::InvalidDesc { raw: utf8.into() });
        }

        // Find the closing ')'.  close_paren_rel is the offset of ')' inside `bytes[1..]`.
        let close_paren_rel = bytes[1..]
            .iter()
            .position(|&b| b == b')')
            .ok_or_else(|| ResolveError::InvalidDesc { raw: utf8.into() })?;
        // Absolute position of ')' in the full string.
        let close_paren_abs = close_paren_rel + 1;

        // Parse parameter descriptors – each is a complete FieldDesc.
        let mut params_desc = Vec::new();
        let mut pos = 1; // right after '('
        while pos < close_paren_abs {
            let len = Self::field_desc_len(&utf8[pos..]);
            let param_str = &utf8[pos..pos + len];
            let field_desc = FieldDesc::from(&param_str.to_string())?;
            params_desc.push(field_desc);
            pos += len;
        }

        // Parse return descriptor
        let ret_start = close_paren_abs + 1; // skip ')'
        if ret_start >= utf8.len() {
            return Err(ResolveError::InvalidDesc { raw: utf8.into() });
        }

        let ret_str = &utf8[ret_start..];
        let ret_desc = if ret_str.as_bytes()[0] == b'V' {
            ReturnDesc::Void
        } else {
            ReturnDesc::Type(FieldDesc::from(&ret_str.to_string())?)
        };

        Ok(MethodDesc {
            raw: SymbolTable::intern(utf8),
            ret_desc,
            params_desc,
        })
    }

    /// Returns the byte length of a field descriptor at the start of `s`.
    fn field_desc_len(s: &str) -> usize {
        let bytes = s.as_bytes();
        let mut pos = 0;
        while pos < bytes.len() && bytes[pos] == b'[' {
            pos += 1;
        }
        if pos >= bytes.len() {
            return pos;
        }
        match bytes[pos] {
            b'L' => {
                // Find the ';'
                bytes[pos..]
                    .iter()
                    .position(|&b| b == b';')
                    .map(|p| pos + p + 1)
                    .unwrap_or(s.len())
            }
            _ => pos + 1, // primitive type
        }
    }
}
