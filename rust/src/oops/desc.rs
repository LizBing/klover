use crate::{
    class_parser::{FieldDescriptor as ParsedFieldDescriptor, FieldType, MethodDescriptor},
    oops::{
        jvalue::JValue,
        symbol_table::{SymbolHandle, SymbolTable},
    },
    runtime::ms_api::{MsAllocator, MsBox},
};

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

impl FieldDesc {
    pub fn raw(&self) -> &SymbolHandle {
        &self.raw
    }

    /// Checks the value kind only; reference assignability requires runtime class metadata.
    pub fn matches_value(&self, value: &JValue) -> bool {
        if self.dimensions > 0 {
            return matches!(value, JValue::Ref(_));
        }

        matches!(
            (&self.elem, value),
            (ElemDesc::Boolean, JValue::Boolean(_))
                | (ElemDesc::Byte, JValue::Byte(_))
                | (ElemDesc::Char, JValue::Char(_))
                | (ElemDesc::Double, JValue::Double(_))
                | (ElemDesc::Float, JValue::Float(_))
                | (ElemDesc::Int, JValue::Int(_))
                | (ElemDesc::Long, JValue::Long(_))
                | (ElemDesc::Short, JValue::Short(_))
                | (ElemDesc::Class(_), JValue::Ref(_))
        )
    }
}

impl From<&ParsedFieldDescriptor> for FieldDesc {
    fn from(value: &ParsedFieldDescriptor) -> Self {
        let raw = SymbolTable::intern(value.raw());

        let elem = match value.element() {
            FieldType::Boolean => ElemDesc::Boolean,
            FieldType::Byte => ElemDesc::Byte,
            FieldType::Char => ElemDesc::Char,
            FieldType::Double => ElemDesc::Double,
            FieldType::Float => ElemDesc::Float,
            FieldType::Int => ElemDesc::Int,
            FieldType::Long => ElemDesc::Long,
            FieldType::Short => ElemDesc::Short,
            FieldType::Object(name) => ElemDesc::Class(SymbolTable::intern(name)),
        };

        Self {
            raw,
            dimensions: value.dimensions() as usize,
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
pub struct MethodDesc<'a> {
    raw: SymbolHandle,
    ret: ReturnDesc,
    args: MsBox<'a, [FieldDesc]>,
}

impl<'a> MethodDesc<'a> {
    pub fn raw(&self) -> &SymbolHandle {
        &self.raw
    }

    pub fn args(&self) -> &[FieldDesc] {
        &self.args
    }

    pub fn build(parsed: &MethodDescriptor, msa: &'a MsAllocator) -> Self {
        let raw = SymbolTable::intern(parsed.raw());
        let ret = match parsed.return_type() {
            None => ReturnDesc::Void,
            Some(fd) => ReturnDesc::Type(FieldDesc::from(fd)),
        };

        let mut args = MsBox::new_uninit_slice(msa, parsed.parameters().len());
        for (i, v) in parsed.parameters().iter().enumerate() {
            args[i].write(FieldDesc::from(v));
        }

        Self {
            raw,
            ret,
            args: unsafe { args.assume_init() },
        }
    }
}
