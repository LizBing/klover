use crate::class_parser::{
    class_reader::ClassReader,
    cp_info::ConstantPoolInfo,
    parse_error::{ClassFileError, ClassFileErrorKind, ClassFileResult},
};

pub struct ExceptionTableEntryInfo {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

impl ExceptionTableEntryInfo {
    fn read(rd: &mut ClassReader) -> ClassFileResult<Self> {
        Ok(Self {
            start_pc: rd.read_u16()?,
            end_pc: rd.read_u16()?,
            handler_pc: rd.read_u16()?,
            catch_type: rd.read_u16()?,
        })
    }
}

pub struct CodeAttrInfo {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<ExceptionTableEntryInfo>,
    pub attrs: Vec<AttrInfo>,
}

impl CodeAttrInfo {
    fn read(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ClassFileResult<Self> {
        let max_stack = rd.read_u16()?;
        let max_locals = rd.read_u16()?;

        let code_len = rd.read_u32()?;
        let code = rd.read(code_len as usize)?.to_vec();

        let et_len = rd.read_u16()?;
        let mut exception_table = Vec::with_capacity(et_len as usize);
        for _ in 0..et_len {
            exception_table.push(ExceptionTableEntryInfo::read(rd)?);
        }

        Ok(Self {
            max_stack,
            max_locals,
            code,
            exception_table,
            attrs: read_attrs(rd, cp)?,
        })
    }
}

pub enum AttrInfo {
    ConstantValue { cp_idx: u16 },

    Code(CodeAttrInfo),

    Unsupported,
}

impl AttrInfo {
    pub(super) fn read(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ClassFileResult<Self> {
        let offset = rd.position();
        
        let name_idx = rd.read_u16()?;        
        let len = rd.read_u32()? as usize;
        let payload = rd.read(len)?;

        let mut pl_rd = ClassReader::new(payload);

        let utf8 = &cp[name_idx as usize];
        let ConstantPoolInfo::Utf8(name) = utf8 else {
            unreachable!()
        };

        match name.as_str() {
            "ConstantValue" => {
                if len != 2 {
                    return Err(ClassFileError {
                        offset,
                        kind: ClassFileErrorKind::InvalidAttributeLength {
                            name_index: name_idx,
                            declared: len
                        }
                    });
                }

                let cp_idx = pl_rd.read_u16()?;

                Ok(Self::ConstantValue { cp_idx })
            }

            "Code" => Ok(Self::Code(CodeAttrInfo::read(&mut pl_rd, cp)?)),

            _ => Ok(Self::Unsupported),
        }
    }
}

pub(super) fn read_attrs(rd: &mut ClassReader, cp: &[ConstantPoolInfo]) -> ClassFileResult<Vec<AttrInfo>> {
    let attrs_count = rd.read_u16()?;
    let mut attrs = Vec::with_capacity(attrs_count as usize);
    for _ in 0..attrs_count {
        attrs.push(AttrInfo::read(rd, cp)?);
    }

    Ok(attrs)
}
