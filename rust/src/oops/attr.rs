use std::{cell::OnceCell, marker::PhantomData};

use crate::{
    class_loader::ms_api::{MSAllocator, MSBox, MSRef}, class_parser::attr_info::{CodeAttrInfo, ExceptionTableEntryInfo}
};

#[derive(Debug)]
pub struct ExceptionTableEntry {
    __: PhantomData<()>,
    
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    /// `None` 表示 catch all（finally 块或 catch_type == 0）。
    pub catch_type: Option<MSRef<ClassCPEntry>>,
}

impl ExceptionTableEntry {
    fn build(info: &ExceptionTableEntryInfo, cp: &[OnceCell<CPEntry>]) -> Self {
        let catch_type = if info.catch_type == 0 {
            None // catch all
        } else {
            let Some(CPEntry::Class(entry)) = cp_slice_get(cp, info.catch_type as usize) else {
                unreachable!()
            };
            unsafe { Some(MSRef::from_raw(entry.into())) }
        };

        Self {
            __: PhantomData,
            start_pc: info.start_pc,
            end_pc: info.end_pc,
            handler_pc: info.handler_pc,
            catch_type,
        }
    }
}

#[derive(Debug)]
pub struct Code {
    __: PhantomData<()>,
    
    pub max_stack: usize,
    pub max_locals: usize,
    pub bytecodes: MSBox<[u8]>,
    pub exception_table: MSBox<[ExceptionTableEntry]>,
}

impl Code {
    pub fn build(
        info: &CodeAttrInfo,
        cp: &[OnceCell<CPEntry>],
        msa: &MSAllocator,
    ) -> Self {
        let code = unsafe {
            let len = info.code.len();
            let uninit = msa.calloc(len);
            let slice = uninit.write_copy_of_slice(&info.code);

            MSBox::from_raw(slice)
        };

        let et = unsafe {
            let len = info.exception_table.len();
            let uninit = msa.calloc(len);

            for (i, v) in info.exception_table.iter().enumerate() {
                let entry = ExceptionTableEntry::build(v, cp);
                uninit[i].write(entry);
            }

            MSBox::from_raw(uninit.assume_init_mut())
        };

        Self {
            __: PhantomData,
            max_stack: info.max_stack as usize,
            max_locals: info.max_locals as usize,
            bytecodes: code,
            exception_table: et,
        }
    }
}

#[derive(Debug)]
pub enum ConstantValue {
    Integer(JInt),
    Float(JFloat),
    Long(JLong),
    Double(JDouble),
    String(MSRef<StringCPEntry>),
}

impl ConstantValue {
    pub fn build(cp_idx: usize, cp: &[OnceCell<CPEntry>]) -> Self {
        match cp[cp_idx].get() {
            Some(CPEntry::Integer(value)) => Self::Integer(*value),
            Some(CPEntry::Float(value)) => Self::Float(*value),
            Some(CPEntry::Long(value)) => Self::Long(*value),
            Some(CPEntry::Double(value)) => Self::Double(*value),
            Some(CPEntry::StringConstant(entry)) => unsafe { Self::String(MSRef::from_raw(entry.into())) },

            _ => unreachable!(),
        }
    }
}
