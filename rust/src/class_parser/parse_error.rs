#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassFileError {
    pub offset: usize,
    pub kind: ClassFileErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassFileErrorKind {
    UnexpectedEof {
        needed: usize,
        remaining: usize,
    },

    InvalidMagic(u32),

    UnsupportedVersion {
        major: u16,
        minor: u16,
    },

    InvalidConstantPoolTag {
        cp_index: u16,
        tag: u8,
    },

    InvalidModifiedUtf8 {
        cp_index: u16,
        bytes: Vec<u8>,
    },

    InvalidAttributeLength {
        name_index: u16,
        declared: usize,
    },

    TrailingBytes {
        remaining: usize,
    },
}

pub type ClassFileResult<T> = Result<T, ClassFileError>;

#[derive(Debug, Clone)]
pub enum CPKind {
    Class,

    Fieldref,

    Methodref,

    InterfaceMethodref,

    Utf8,

    ConstantValue,

    NameAndType,

    MethodHandle,

    MethodType,

    InvokeDynamic,

    Unusable,
}

#[derive(Debug, Clone)]
pub enum ImageError {
    InvalidCPIndex(u16),

    CPKindMismatched {
        index: u16,
        expected: CPKind,
    },

    InvalidRefKind(u8),

    InvalidMemberRefName(String),

    InvalidDesc(String),
}

pub type ImageResult<T> = Result<T, ImageError>;

#[derive(Debug, Clone)]
pub enum ParseError {
    ClassFile(ClassFileError),
    Image(ImageError),
}
