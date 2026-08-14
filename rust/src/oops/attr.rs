use std::cell::OnceCell;

use crate::{
    class_loader::ms_api::{MSAllocator, MSBox, MSRef}, class_parser::attr_info::{CodeAttrInfo, ExceptionTableEntryInfo}, oops::{
        cp_entry::{CPEntry, ClassCPEntry, StringCPEntry}, normal_klass::cp_slice_get,
    },
};

#[derive(Debug)]
pub struct ExceptionTableEntry {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    /// `None` 表示 catch all（finally 块或 catch_type == 0）。
    catch_type: Option<MSRef<ClassCPEntry>>,
}

impl ExceptionTableEntry {
    fn build(info: &ExceptionTableEntryInfo, cp: &[OnceCell<CPEntry>]) -> Self {
        let catch_type = if info.catch_type == 0 {
            None // catch all
        } else {
            let ct = match cp_slice_get(cp, info.catch_type as usize) {
                Some(CPEntry::Class(entry)) => entry,
                _ => unreachable!(),
            };
            unsafe { Some(MSRef::from_raw(ct.into())) }
        };

        Self {
            start_pc: info.start_pc,
            end_pc: info.end_pc,
            handler_pc: info.handler_pc,
            catch_type,
        }
    }

    /// 异常处理器的覆盖范围起点（bci，含）。
    pub fn start_pc(&self) -> u16 {
        self.start_pc
    }

    /// 异常处理器的覆盖范围终点（bci，不含）。
    pub fn end_pc(&self) -> u16 {
        self.end_pc
    }

    /// handler 的起始 bci。
    pub fn handler_pc(&self) -> u16 {
        self.handler_pc
    }

    /// `None` 表示 catch all；`Some` 表示只捕获指定类及其子类。
    pub fn catch_type(&self) -> Option<&MSRef<ClassCPEntry>> {
        self.catch_type.as_ref()
    }
}

#[derive(Debug)]
pub struct Code {
    max_stack: usize,
    max_locals: usize,
    bytecodes: MSBox<[u8]>,
    exception_table: MSBox<[ExceptionTableEntry]>,
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
            max_stack: info.max_stack as usize,
            max_locals: info.max_locals as usize,
            bytecodes: code,
            exception_table: et,
        }
    }
}

impl Code {
    pub fn max_stack(&self) -> usize {
        self.max_stack
    }

    pub fn max_locals(&self) -> usize {
        self.max_locals
    }

    pub fn bytecodes(&self) -> &[u8] {
        &self.bytecodes
    }

    pub fn get_exception_table_entry(&self, idx: usize) -> &ExceptionTableEntry {
        &self.exception_table[idx]
    }
}

#[derive(Debug)]
pub enum ConstantValue {
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
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
