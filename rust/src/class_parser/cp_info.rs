use crate::class_parser::{class_reader::ClassReader, parse_error::{ParseError, ParseResult}};

pub enum ReferenceKind {}

pub enum ConstantPoolInfo {
    ClassInfo {
        name_index: u16     // Utf8Info
    },

    FieldrefInfo {
        class_index: u16,   // ClassInfo
        name_and_type_index: u16    // NameAndTypeInfo
    },
    
    MethodrefInfo {
        class_index: u16,   // ClassInfo
        name_and_type_index: u16    // NameAndTypeInfo
    },
    
    InterfaceMethodrefInfo {
        class_index: u16,   // ClassInfo
        name_and_type_index: u16    // NameAndTypeInfo
    },

    StringInfo {
        string_index: u16   // Utf8Info
    },

    IntegerInfo {
        value: i32
    },
    
    FloatInfo {
        value: f32
    },

    LongInfo {
        value: i64
    },

    DoubleInfo {
        value: f64
    },

    NameAndTypeInfo {
        name_index: u16,    // Utf8Info
        desc_index: u16,    // Utf8Info
    },

    Utf8Info {
        utf8: String
    },

    MethodHandleInfo {
        ref_kind: ReferenceKind,
        ref_index: u16,
    },
    
    MethodTypeInfo {
        desc_index: u16     // Utf8Info
    },

    DynamicInfo {
        bs_method_attr_index: u16,
        name_and_type_index: u16    // NameAndTypeInfo
    },
    
    InvokeDynamicInfo {
        bs_method_attr_index: u16,
        name_and_type_index: u16    // NameAndTypeInfo
    },

    ModuleInfo {
        name_index: u16     // Utf8Info
    },
    
    PackageInfo {
        name_index: u16     // Utf8Info
    }
}

impl ConstantPoolInfo {
    pub fn read(rd: &mut ClassReader) -> ParseResult<Self> {
        let tag = rd.read_u8()?;
        
        let res = match tag {
            7 => Self::ClassInfo { name_index: rd.read_u16()? },
            9 => Self::FieldrefInfo { class_index: rd.read_u16()?, name_and_type_index: rd.read_u16()? },
            10 => Self::MethodrefInfo { class_index: rd.read_u16()?, name_and_type_index: rd.read_u16()? },
            11 => Self::InterfaceMethodrefInfo { class_index: rd.read_u16()?, name_and_type_index: rd.read_u16()? },
            
            _ => return Err(ParseError::InvalidCPTag(tag))
        };

        Ok(res)
    }
}

