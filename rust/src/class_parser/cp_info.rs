use super::{
    class_reader::ClassReader,
    parse_error::{ParseError, ParseResult},
};

#[derive(Debug)]
pub enum ConstantPoolInfo {
    ClassInfo {
        name_index: u16, // Utf8Info
    },

    FieldrefInfo {
        class_index: u16,         // ClassInfo
        name_and_type_index: u16, // NameAndTypeInfo
    },

    MethodrefInfo {
        class_index: u16,         // ClassInfo
        name_and_type_index: u16, // NameAndTypeInfo
    },

    InterfaceMethodrefInfo {
        class_index: u16,         // ClassInfo
        name_and_type_index: u16, // NameAndTypeInfo
    },

    StringInfo {
        string_index: u16, // Utf8Info
    },

    IntegerInfo {
        value: i32,
    },

    FloatInfo {
        value: f32,
    },

    LongInfo {
        value: i64,
    },

    DoubleInfo {
        value: f64,
    },

    NameAndTypeInfo {
        name_index: u16, // Utf8Info
        desc_index: u16, // Utf8Info
    },

    // best-effort UTF-8
    Utf8Info {
        utf8: String,
    },

    /// JVM spec 4.4.5: Long and Double occupy two consecutive slots.
    /// The second slot (n+1) is unusable and must never be referenced.
    Unusable,
}

impl ConstantPoolInfo {
    pub fn read(rd: &mut ClassReader) -> ParseResult<Self> {
        let tag = rd.read_u8()?;

        let res = match tag {
            7 => Self::ClassInfo {
                name_index: rd.read_u16()?,
            },
            9 => Self::FieldrefInfo {
                class_index: rd.read_u16()?,
                name_and_type_index: rd.read_u16()?,
            },
            10 => Self::MethodrefInfo {
                class_index: rd.read_u16()?,
                name_and_type_index: rd.read_u16()?,
            },
            11 => Self::InterfaceMethodrefInfo {
                class_index: rd.read_u16()?,
                name_and_type_index: rd.read_u16()?,
            },
            8 => Self::StringInfo {
                string_index: rd.read_u16()?,
            },
            3 => Self::IntegerInfo {
                value: rd.read_i32()?,
            },
            4 => Self::FloatInfo {
                value: rd.read_f32()?,
            },
            5 => Self::LongInfo {
                value: rd.read_i64()?,
            },
            6 => Self::DoubleInfo {
                value: rd.read_f64()?,
            },
            12 => Self::NameAndTypeInfo {
                name_index: rd.read_u16()?,
                desc_index: rd.read_u16()?,
            },
            1 => {
                let len = rd.read_u16()? as usize;
                let raw = rd.read(len)?;
                let utf8 = match String::from_utf8(Vec::from(raw)) {
                    Ok(x) => x,
                    Err(_) => return Err(ParseError::InvalidUtf8(Vec::from(raw))),
                };

                Self::Utf8Info { utf8 }
            },

            // Ignore for now.
            15 | 16 | 18 => return Err(ParseError::UnsupportedCPTag(tag)),

            _ => return Err(ParseError::InvalidCPTag(tag)),
        };

        Ok(res)
    }
}
