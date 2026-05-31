use super::{class_reader::ClassReader, parse_error::ParseResult};

#[derive(Debug)]
pub enum AttrInfo {
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
        exception_table: Vec<ExceptionTableEntry>,
        attrs: Vec<AttrInfo>,
    },

    ConstantValue {
        const_value_index: u16,
    },

    Exceptions {
        exception_index_table: Vec<u16>,
    },

    Synthetic,

    Deprecated,

    Signature {
        signature_index: u16,
    },

    SourceFile {
        sourcefile_index: u16,
    },

    LineNumberTable {
        line_number_table: Vec<LineNumberTableEntry>,
    },

    LocalVariableTable {
        local_variable_table: Vec<LocalVariableTableEntry>,
    },

    BootstrapMethods {
        bootstrap_methods: Vec<BootstrapMethod>,
    },

    StackMapTable {
        entries: Vec<u8>,
    },

    NestHost {
        host_class_index: u16,
    },

    NestMembers {
        classes: Vec<u16>,
    },

    Unparsed {
        name_index: u16,
        data: Vec<u8>,
    },
}

#[derive(Debug)]
pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

#[derive(Debug)]
pub struct LineNumberTableEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Debug)]
pub struct LocalVariableTableEntry {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub desc_index: u16,
    pub index: u16,
}

#[derive(Debug)]
pub struct BootstrapMethod {
    pub bootstrap_method_ref: u16,
    pub bootstrap_arguments: Vec<u16>,
}

impl AttrInfo {
    pub fn read(rd: &mut ClassReader) -> ParseResult<Self> {
        let name_index = rd.read_u16()?;
        let len = rd.read_u32()? as usize;
        let raw = rd.read(len)?.to_vec();

        // Note: we cannot resolve the attribute name from the constant pool here
        // since we only have the ClassReader. The caller must resolve name_index
        // against the constant pool to determine which attribute variant this is.
        Ok(Self::Unparsed {
            name_index,
            data: raw,
        })
    }
}
