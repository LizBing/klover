use std::{ops::Add, ptr};

use crate::interpreter::interpreter::{DStackSlot, Registers, StackSlot};

const fn slots_of<T>() -> usize {
    size_of::<T>() / size_of::<StackSlot>()
}

fn push<T>(regs: &mut Registers, v: T) {
    unsafe {
        regs.sp = regs.sp.sub(slots_of::<T>());
        *(regs.sp as *mut T) = v;
    }
}

fn pop<T: Copy>(regs: &mut Registers) -> T {
    unsafe {
        let res = *(regs.sp as *const T);
        regs.sp = regs.sp.add(slots_of::<T>());

        res
    }
}

#[inline]
pub fn get_local<T: Copy>(regs: &Registers, idx: usize) -> T {
    let locals_start = unsafe { (regs.bp as *const StackSlot).sub((*regs.bp).code().max_locals as _) };
    let addr = unsafe { locals_start.add(idx) as *const T };

    unsafe { *addr }
}

fn nop(_: &mut Registers) { /* no op */ }

fn local_load_n<const N: usize, T: Copy>(regs: &mut Registers) {
    let value: T = get_local(regs, N);
    push(regs, value);
}

fn add<T: Copy + Add>(regs: &mut Registers) {
    let x: T = pop(regs);
    let y: T = pop(regs);
    push(regs, x + y);
}



fn type_return<T: Copy>(regs: &mut Registers) {
    unimplemented!()
}

pub type InsFnType = fn(&mut Registers);

const INSTRUCTION_COUNT: usize = 202;
pub static INSTRUCTION_TABLE: [InsFnType; INSTRUCTION_COUNT] = [
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    local_load_n::<0, i32>,
    local_load_n::<1, i32>,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    add::<i32>,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    type_return::<i32>,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
    nop,
];
