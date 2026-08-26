use crate::class_parser::parse_error::ClassFileErrorKind;

use super::{
    class_reader::ClassReader,
    parse_error::{ClassFileError, ClassFileResult},
};

#[derive(Debug)]
pub enum ConstantPoolInfo {
    Class {
        name_index: u16, // Utf8Info
    },

    Fieldref {
        class_index: u16,         // ClassInfo
        name_and_type_index: u16, // NameAndTypeInfo
    },

    Methodref {
        class_index: u16,         // ClassInfo
        name_and_type_index: u16, // NameAndTypeInfo
    },

    InterfaceMethodref {
        class_index: u16,         // ClassInfo
        name_and_type_index: u16, // NameAndTypeInfo
    },

    String {
        string_index: u16, // Utf8Info
    },

    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Utf8(String),

    NameAndType {
        name_index: u16, // Utf8Info
        desc_index: u16, // Utf8Info
    },

    MethodHandle {
        ref_kind: u8,
        ref_index: u16,
    },

    MethodType {
       desc_index: u16, 
    },

    InvokeDynamic {
        bs_method_attr_index: u16,
        name_and_type_index: u16,
    },

    /// JVM spec 4.4.5: Long and Double occupy two consecutive slots.
    /// The second slot (n+1) is unusable and must never be referenced.
    Unusable,
}

impl ConstantPoolInfo {
    pub fn read(rd: &mut ClassReader, cp_index: u16) -> ClassFileResult<Self> {
        let offset = rd.position();
        let tag = rd.read_u8()?;

        let res = match tag {
            7 => Self::Class {
                name_index: rd.read_u16()?,
            },
            
            9 => Self::Fieldref {
                class_index: rd.read_u16()?,
                name_and_type_index: rd.read_u16()?,
            },
            10 => Self::Methodref {
                class_index: rd.read_u16()?,
                name_and_type_index: rd.read_u16()?,
            },
            11 => Self::InterfaceMethodref {
                class_index: rd.read_u16()?,
                name_and_type_index: rd.read_u16()?,
            },
            
            8 => Self::String {
                string_index: rd.read_u16()?,
            },
            3 => Self::Integer(rd.read_i32()?),
            4 => Self::Float(rd.read_f32()?),
            5 => Self::Long(rd.read_i64()?),
            6 => Self::Double(rd.read_f64()?),
            
            12 => Self::NameAndType {
                name_index: rd.read_u16()?,
                desc_index: rd.read_u16()?,
            },
            
            1 => {
                let len = rd.read_u16()? as usize;
                let raw = rd.read(len)?;
                let utf8 = cesu8::from_java_cesu8(raw).map_err(|_| ClassFileError {
                    offset,
                    kind: ClassFileErrorKind::InvalidModifiedUtf8 {
                        cp_index,
                        bytes: raw.into()
                    }
                })?;

                Self::Utf8(utf8.into())
            },

            // Ignore for now.
            // 15 | 16 | 18 => return Err(ParseError::UnsupportedCPTag(tag)),
            // 
            15 => Self::MethodHandle {
                ref_kind: rd.read_u8()?,
                ref_index: rd.read_u16()?,
            },

            16 => Self::MethodType {
                desc_index: rd.read_u16()?,
            },

            18 => Self::InvokeDynamic {
                bs_method_attr_index: rd.read_u16()?,
                name_and_type_index: rd.read_u16()?,
            },

            _ => return Err(ClassFileError {
                offset,
                kind: ClassFileErrorKind::InvalidConstantPoolTag { cp_index, tag }
            }),
        };

        Ok(res)
    }
}
