use crate::{runtime::ms_api::{MsAllocator, MsBox}, oops::symbol_table::{SymbolHandle, SymbolTable}};

#[derive(Debug)]
pub enum ElemDesc {
    Boolean,
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Class(SymbolHandle),
}

#[derive(Debug)]
pub struct FieldDesc {
    raw: SymbolHandle,
    dimensions: usize,
    elem: ElemDesc,
}

use cafebabe::descriptors::FieldType::*;
impl From<&cafebabe::descriptors::FieldDescriptor<'_>> for FieldDesc {
    fn from(value: &cafebabe::descriptors::FieldDescriptor) -> Self {
        let raw = SymbolTable::intern(&value.to_string());

        let elem = match &value.field_type {
            Boolean => ElemDesc::Boolean,
            Byte => ElemDesc::Byte,
            Char => ElemDesc::Char,
            Double => ElemDesc::Double,
            Float => ElemDesc::Float,
            Integer => ElemDesc::Int,
            Long => ElemDesc::Long,
            Short => ElemDesc::Short,
            Object(cn) => ElemDesc::Class(SymbolTable::intern(cn)),
        };

        Self {
            raw,
            dimensions: value.dimensions as usize,
            elem,
        }
    }
}

#[derive(Debug)]
pub enum ReturnDesc {
    Void,
    Type(FieldDesc),
}

#[derive(Debug)]
pub struct MethodDesc {
    raw: SymbolHandle,
    ret: ReturnDesc,
    args: MsBox<[FieldDesc]>,
}

use cafebabe::descriptors::ReturnDescriptor::*;
impl MethodDesc {
    pub fn raw(&self) -> &SymbolHandle {
        &self.raw
    }

    pub fn build(parsed: &cafebabe::descriptors::MethodDescriptor, msa: &MsAllocator) -> Self {
        let raw = SymbolTable::intern(&parsed.to_string());
        let ret = match &parsed.return_type {
            Void => ReturnDesc::Void,
            Return(fd) => ReturnDesc::Type(FieldDesc::from(fd)),
        };

        let args = msa.calloc(parsed.parameters.len());
        for (i, v) in parsed.parameters.iter().enumerate() {
            args[i].write(FieldDesc::from(v));
        }

        unsafe {
            Self {
                raw,
                ret,
                args: MsBox::from_raw(args.assume_init_mut()),
            }
        }
    }
}
