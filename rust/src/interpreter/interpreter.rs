use std::{marker::PhantomData, mem::MaybeUninit, ptr::{self, NonNull}};

use crate::oops::{attr::CodeAttr, method::Method, normal_klass::NormalKlass};

pub type StackSlot = i32;
pub type DStackSlot = i64;

pub struct Frame {
    ctx: Registers,
    
    pub klass: NonNull<NormalKlass>,
    pub method: NonNull<Method>
}

impl Frame {
    pub fn code(&self) -> &CodeAttr {
        unsafe { self.method.as_ref().code.as_ref().unwrap_unchecked() }
    }
}

pub(super) struct Registers {
    __: PhantomData<()>,
    
    pub bp: *mut Frame,
    pub sp: *mut StackSlot,
    pub pc: *const u8
}

impl Registers {
    pub fn store(dst: &mut Self, src: &Self) {
        unsafe {
            ptr::copy(src, dst, 1);
        }
    }
}

pub struct Interpreter {
    regs: Registers,
    stack: Box<[MaybeUninit<u8>]>
}

impl Interpreter {}
