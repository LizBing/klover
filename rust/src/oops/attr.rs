use std::cell::OnceCell;

use crate::{
    class_loader::ms_api::{MSAllocator, MSBox, MSRef}, class_parser::attr_info::{BootstrapMethodInfo, CodeAttrInfo, ExceptionTableEntryInfo}, oops::{
        cp_entry::{CPEntry, ClassCPEntry, Loadable, MethodHandleEntry, StringCPEntry}, normal_klass::cp_slice_get, resolve_error::{ResolveError, ResolveResult}, symbol_table::SymbolHandle,
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
    fn build(info: &ExceptionTableEntryInfo, cp: &[OnceCell<CPEntry>]) -> ResolveResult<Self> {
        let catch_type = if info.catch_type == 0 {
            None // catch all
        } else {
            let ct = match cp_slice_get(cp, info.catch_type as usize) {
                Some(CPEntry::Class(entry)) => entry,
                _ => return Err(ResolveError::MismatchCPType),
            };
            Some(ct.into())
        };

        Ok(Self {
            start_pc: info.start_pc,
            end_pc: info.end_pc,
            handler_pc: info.handler_pc,
            catch_type,
        })
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
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: MSBox<[u8]>,
    pub exception_table: MSBox<[ExceptionTableEntry]>,
}

impl Code {
    pub fn build(
        info: &CodeAttrInfo,
        cp: &[OnceCell<CPEntry>],
        msa: &MSAllocator,
    ) -> ResolveResult<Self> {
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
                let entry = ExceptionTableEntry::build(v, cp)?;
                uninit[i].write(entry);
            }

            MSBox::from_raw(uninit.assume_init_mut())
        };

        Ok(Self {
            max_stack: info.max_stack,
            max_locals: info.max_locals,
            code,
            exception_table: et,
        })
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
    pub fn build(cp_idx: usize, cp: &[OnceCell<CPEntry>]) -> ResolveResult<Self> {
        match cp[cp_idx].get() {
            Some(CPEntry::Integer(value)) => Ok(Self::Integer(*value)),
            Some(CPEntry::Float(value)) => Ok(Self::Float(*value)),
            Some(CPEntry::Long(value)) => Ok(Self::Long(*value)),
            Some(CPEntry::Double(value)) => Ok(Self::Double(*value)),
            Some(CPEntry::StringConstant(entry)) => Ok(Self::String(entry.into())),

            _ => Err(ResolveError::MismatchCPType),
        }
    }
}

pub fn build_permitted_subclasses(parsed: &[u16], cp: &[OnceCell<CPEntry>], msa: &MSAllocator) -> ResolveResult<MSBox<[MSRef<ClassCPEntry>]>> {
    let len = parsed.len();
    let uninit = msa.calloc(len);

    for i in 0..len {
        let idx = parsed[i] as usize;
        let cp_ref = match cp[idx].get() {
            Some(CPEntry::Class(x)) => x.into(),
            _ => return Err(ResolveError::MismatchCPType)
        };

        uninit[i].write(cp_ref);
    }

    unsafe { Ok(MSBox::from_raw(uninit.assume_init_mut())) }
}

#[derive(Debug)]
pub struct BootstrapMethod {
    bs_method_ref: MSRef<MethodHandleEntry>,
    args: MSBox<[Loadable]>
}

impl BootstrapMethod {
    fn build(info: &BootstrapMethodInfo, cp: &[OnceCell<CPEntry>], msa: &MSAllocator) -> ResolveResult<Self> {
        let bsm_ref = match cp[info.bs_method_ref as usize].get() {
            Some(CPEntry::MethodHandle(x)) => x.into(),
            _ => return Err(ResolveError::MismatchCPType)
        };
        
        let len = info.bs_arguments.len();
        let uninit = msa.calloc(len);

        for (i, v) in info.bs_arguments.iter().enumerate() {
            let arg = match cp[*v as usize].get() {
                Some(CPEntry::Integer(x)) => Loadable::Integer(*x),
                Some(CPEntry::Float(x)) => Loadable::Float(*x),
                Some(CPEntry::Long(x)) => Loadable::Long(*x),
                Some(CPEntry::Double(x)) => Loadable::Double(*x),
                Some(CPEntry::Class(x)) => Loadable::Class(x.into()),
                Some(CPEntry::StringConstant(x)) => Loadable::StringLoadable(x.into()),
                Some(CPEntry::MethodHandle(x)) => Loadable::MethodHandle(x.into()),
                Some(CPEntry::MethodType(x)) => Loadable::MethodType(x.clone()),
                Some(CPEntry::Dynamic(x)) => Loadable::Dynamic(x.into()),

                _ => return Err(ResolveError::MismatchCPType),
            };

            uninit[i].write(arg);
        }

        Ok(Self {
            bs_method_ref: bsm_ref,
            args: unsafe { MSBox::from_raw(uninit.assume_init_mut()) }
        })
    }
}

pub fn build_bs_methods(infos: &[BootstrapMethodInfo], cp: &[OnceCell<CPEntry>], msa: &MSAllocator) -> ResolveResult<MSBox<[BootstrapMethod]>> {
    let len = infos.len();
    let uninit = msa.calloc(len);

    for (i, v) in infos.iter().enumerate() {
        let bsm = BootstrapMethod::build(v, cp, msa)?;
        uninit[i].write(bsm);
    }

    unsafe { Ok(MSBox::from_raw(uninit.assume_init_mut())) }
}

pub fn build_nest_members(idxes: &[u16], cp: &[OnceCell<CPEntry>], msa: &MSAllocator) -> ResolveResult<MSBox<[MSRef<ClassCPEntry>]>> {
    let len = idxes.len();
    let uninit = msa.calloc(len);

    for (i, v) in idxes.iter().enumerate() {
        let cp_ref = match cp[*v as usize].get() {
            Some(CPEntry::Class(x)) => x.into(),
            _ => return Err(ResolveError::MismatchCPType)
        };

        uninit[i].write(cp_ref);
    }

    unsafe { Ok(MSBox::from_raw(uninit.assume_init_mut())) }
}
