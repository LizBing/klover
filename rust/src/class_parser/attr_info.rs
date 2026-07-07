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

fn read_u16_vec(rd: &mut ClassReader, count: usize) -> ParseResult<Vec<u16>> {
    let mut res = Vec::new();

    for _ in 0..count {
        res.push(rd.read_u16()?);
    }

    Ok(res)
}

pub struct BootstrapMethodInfo {
    pub bs_method_ref: u16,
    pub bs_arguments: Vec<u16>
}

impl BootstrapMethodInfo {
    fn read(rd: &mut ClassReader) -> ParseResult<Self> {
        let bs_method_ref = rd.read_u16()?;
        let count = rd.read_u16()?;
        let bs_arguments = read_u16_vec(rd, count as usize)?;

        Ok(Self {
            bs_method_ref,
            bs_arguments
        })
    }
}

pub enum AttrInfo {
    ConstantValue { cp_idx: u16 },
    
    Code(CodeAttrInfo),

    PermittedSubclasses { cp_idxes: Vec<u16> },

    BootstrapMethods(Vec<BootstrapMethodInfo>),

    NestHost { cp_idx: u16 },

    NestMembers { cp_idxes: Vec<u16> },
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
            
            "Code" => Ok(Some(Self::Code(CodeAttrInfo::read(&mut pl_rd, cp)?))),
            
            "PermittedSubclasses" => {
                let count = pl_rd.read_u16()?;
                let res = read_u16_vec(&mut pl_rd, count as usize)?;

                Ok(Some(Self::PermittedSubclasses { cp_idxes: res }))
            }

            "BootstrapMethods" => {
                let count = pl_rd.read_u16()?;
                let mut methods = Vec::new();

                for _ in 0..count {
                    methods.push(BootstrapMethodInfo::read(&mut pl_rd)?);
                }

                Ok(Some(Self::BootstrapMethods(methods)))
            }

            "NestHost" => Ok(Some(Self::NestHost { cp_idx: pl_rd.read_u16()? })),

            "NestMembers" => {
                let count = pl_rd.read_u16()?;
                let res = read_u16_vec(&mut pl_rd, count as usize)?;

                Ok(Some(Self::PermittedSubclasses { cp_idxes: res }))
            }

            _ => Ok(None)
        }
    }
}
