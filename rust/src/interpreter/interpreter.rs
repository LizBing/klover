use std::ptr::{NonNull, null_mut};

use crate::oops::{attr::CodeAttr, method::Method, normal_klass::NormalKlass};

pub type StackSlot = i32;
pub type DStackSlot = i64;

pub struct Frame {
    pub ctx: Registers,
    pub klass: NonNull<NormalKlass>,
    pub method: NonNull<Method>
}

impl Frame {
    pub fn code(&self) -> &CodeAttr {
        unsafe { self.method.as_ref().code.as_ref().unwrap_unchecked() }
    }
}

pub struct Registers {
    pub bp: *mut Frame,
    pub sp: *mut StackSlot,
    pub pc: *const u8
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            bp: null_mut(),
            sp: null_mut(),
            pc: null_mut()
        }
    }
}

#[inline]
pub fn get_local<T: Copy>(regs: &Registers, idx: usize) -> T {
    let locals_start = unsafe { (regs.bp as *const StackSlot).sub((*regs.bp).code().max_locals as _) };
    let addr = unsafe { locals_start.add(idx) as *const T };

    unsafe { *addr }
}
