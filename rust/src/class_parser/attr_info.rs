use crate::class_parser::{class_file::read_attrs, class_reader::ClassReader, cp_info::ConstantPoolInfo, parse_error::{ParseError, ParseResult}};

pub struct ExceptionTableEntryInfo {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

impl ExceptionTableEntryInfo {
    fn read(rd: &mut ClassReader) -> ParseResult<Self> {
        Ok(Self {
            start_pc: rd.read_u16()?,
            end_pc: rd.read_u16()?,
            handler_pc: rd.read_u16()?,
            catch_type: rd.read_u16()?
        })
    }
}

pub struct CodeAttrInfo {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<ExceptionTableEntryInfo>,
    pub attrs: Vec<AttrInfo>
}

impl CodeAttrInfo {
    fn read(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ParseResult<Self> {
        let max_stack = rd.read_u16()?;
        let max_locals = rd.read_u16()?;

        let code_len = rd.read_u32()?;
        let code = rd.read(code_len as _)?.to_vec();

        let et_len = rd.read_u16()?;
        let mut exception_table = Vec::with_capacity(et_len as _);
        for _ in 0..et_len {
            exception_table.push(ExceptionTableEntryInfo::read(rd)?);
        }

        Ok(Self {
            max_stack,
            max_locals,
            code,
            exception_table,
            attrs: read_attrs(rd, cp)?
        })
    }
}

pub enum AttrInfo {
    ConstantValue { cp_idx: u16 },
    
    Code { info: CodeAttrInfo },
}

impl AttrInfo {
    pub fn read(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ParseResult<Option<Self>> {
        let name_idx = rd.read_u16()?;
        let len = rd.read_u32()?;
        let payload = rd.read(len as _)?;

        let mut pl_rd = ClassReader::new(payload);

        let utf8_info = &cp[name_idx as usize];
        let name = match utf8_info {
            ConstantPoolInfo::Utf8Info { utf8 } => utf8,
            _ => return Err(ParseError::InvalidCPType)
        };

        match name.as_str() {
            "ConstantValue" => Ok(Some(Self::ConstantValue { cp_idx: pl_rd.read_u16()? })),
            "Code" => Ok(Some(Self::Code { info: CodeAttrInfo::read(&mut pl_rd, cp)? })),

            _ => Ok(None)
        }
    }
}
