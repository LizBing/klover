use std::{
    ptr::{self, NonNull},
    slice,
};

use crate::{
    class_loader::ms_api::{MSAllocator, MSRef},
    class_parser::attr_info::{CodeAttrInfo, ExceptionTableEntryInfo},
    oops::{
        cp_entry::{CPEntry, ClassCPEntry, StringCPEntry}, normal_klass::cp_slice_get, resolve_error::{ResolveError, ResolveResult}
    },
};

pub struct ExceptionTableEntry {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: MSRef<ClassCPEntry>,
}

impl ExceptionTableEntry {
    fn from(info: &ExceptionTableEntryInfo, cp: &[Option<CPEntry>]) -> ResolveResult<Self> {
        let ct = unsafe {
            match cp_slice_get(cp, info.catch_type as usize) {
                CPEntry::Class { entry } => entry,
                _ => return Err(ResolveError::MismatchCPType),
            }
        };

        Ok(Self {
            start_pc: info.start_pc,
            end_pc: info.end_pc,
            handler_pc: info.handler_pc,
            catch_type: ct,
        })
    }
}

pub struct CodeAttr {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: NonNull<[u8]>,
    pub exception_table: NonNull<[ExceptionTableEntry]>,
}

impl CodeAttr {
    pub fn from(info: &CodeAttrInfo, cp: &[Option<CPEntry>], msa: &MSAllocator) -> ResolveResult<Self> {
        let code = unsafe {
            let len = info.code.len();
            let mem = msa.alloc(len);
            ptr::copy(info.code.as_ptr(), mem, len);

            NonNull::new_unchecked(slice::from_raw_parts_mut(mem, len))
        };

        let et = unsafe {
            let len = info.exception_table.len();
            let mem = msa.calloc(size_of::<ExceptionTableEntry>(), len);
            slice::from_raw_parts_mut(mem, len)
        };

        let mut i = 0;
        for n in &info.exception_table {
            et[i] = ExceptionTableEntry::from(n, cp)?;
            i += 1;
        }

        Ok(Self {
            max_stack: info.max_stack,
            max_locals: info.max_locals,
            code,
            exception_table: et.into(),
        })
    }
}

pub enum ConstantValueAttr {
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    String(NonNull<StringCPEntry>),
}

impl ConstantValueAttr {
    pub fn from(cp_idx: usize, cp: &[Option<CPEntry>]) -> ResolveResult<Self> {
        let entry = unsafe { cp[cp_idx].as_ref().unwrap_unchecked() };

        match entry {
            CPEntry::Integer { value } => Ok(Self::Integer(*value)),
            CPEntry::Float { value } => Ok(Self::Float(*value)),
            CPEntry::Long { value } => Ok(Self::Long(*value)),
            CPEntry::Double { value } => Ok(Self::Double(*value)),
            CPEntry::StringConstant { entry } => {
                let ptr = entry as *const StringCPEntry as *mut StringCPEntry;
                Ok(Self::String(unsafe { NonNull::new_unchecked(ptr) }))
            }
            _ => Err(ResolveError::MismatchCPType),
        }
    }
}
