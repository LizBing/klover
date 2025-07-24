/*
 * Copyright 2025 Lei Zaakjyu
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::{ffi::c_void, ptr::null_mut};

use crate::{align_up, memory::{allocation::c_heap_alloc, mem_region::MemRegion, virt_space::VirtSpace}, utils::global_defs::{addr_cast, address, word_t}};

pub type slot_t = word_t;

struct Frame {
    stored_pc: u32,
    stored_bp: *mut Frame,

    callback: address
}

impl Frame {
    fn store(&mut self, pc: u32, bp: *mut Frame, callback: address) {
        self.stored_pc = pc;
        self.stored_bp = bp;
        self.callback = callback;
    }
}

impl Clone for Frame {
    fn clone(&self) -> Self {
        Self {
            stored_pc: 0,
            stored_bp: null_mut(),
            callback: self.callback
        }
    }
}

pub struct VMRegiters {
    pub pc: u32,

    sp: address,
    bp: *mut Frame
}

impl VMRegiters {
    fn new() -> Self {
        VMRegiters { pc: 0, sp: 0, bp: null_mut() }
    }
}

pub struct Context {
    pub _regs: VMRegiters,
    _stack: MemRegion
}

impl Context {
    pub fn new(stack_size: usize) -> Self {
        Self {
            _regs: VMRegiters::new(),
            _stack: c_heap_alloc(align_up!(stack_size, VirtSpace::page_size())).unwrap()
        }
    }

    pub const fn size_of_slots(n: usize) -> usize {
        size_of::<slot_t>() * n
    }

    // Ensure reachable.
    pub fn reserve(&mut self, slots: usize) -> bool {
        let size = Self::size_of_slots(slots);
        let regs = &mut self._regs;

        self._stack.contains(regs.sp - size)
    }

    pub fn alloca(&mut self, slots: usize, zeroing: bool) -> address {
        let size = Self::size_of_slots(slots);
        let regs = &mut self._regs;
        
        if !self._stack.contains(regs.sp - size) {
            return 0;
        }
        regs.sp -= size;
        let res = regs.sp;

        if zeroing {
            unsafe { MemRegion::with_size(res, size).memset(0); } 
        }

        res
    }

    pub fn try_free(&mut self, addr: address, slots: usize) -> bool {
        let size = Self::size_of_slots(slots);
        
        let regs = &mut self._regs;
        if regs.sp != addr && !self._stack.contains(regs.sp + size) { return false; }

        regs.sp += size;

        true
    }

    pub fn create_frame(this: *mut Context, callback: address) -> bool {
        unsafe {
            let regs = &mut (*this)._regs;

            let mem = (*this).alloca(size_of::<Frame>(), false);
            if mem == 0 { return false; }
            let new_frame = addr_cast::<Frame>(mem);

            new_frame.store(regs.pc, regs.bp, callback);
            regs.bp = new_frame;
        }

        true
    }

    // helper
    fn cal_unwind_sp_offs(bp: *const Frame) -> address {
        bp as address + size_of::<Frame>()
    }

    pub fn unwind(this: *mut Context) -> address {
        unsafe {
            let regs = &mut (*this)._regs;

            let frame = regs.bp;
            if frame == null_mut() { return 0; }

            regs.sp = Self::cal_unwind_sp_offs(regs.bp);
            regs.bp = (*frame).stored_bp;
            regs.pc = (*frame).stored_pc;

            (*frame).callback
        }
    }

    // unsafe
    pub fn push_1slot<T: Copy>(&mut self, n: T) {
        let regs = &mut self._regs;
        regs.sp -= Self::size_of_slots(1);

        unsafe { *(regs.sp as *mut c_void as *mut T) = n; }
    }

    pub fn push_2slots<T: Copy>(&mut self, n: T) {
        let regs = &mut self._regs;
        regs.sp -= Self::size_of_slots(2);

        unsafe { *(regs.sp as *mut c_void as *mut T) = n; }
    }

    pub fn pop_1slot<T: Copy>(&mut self) -> T {
        let regs = &mut self._regs;

        let res = regs.sp;
        regs.sp += Self::size_of_slots(1);        

        unsafe { *(res as *const c_void as *const T) }
    }

    pub fn pop_2slots<T: Copy>(&mut self) -> T {
        let regs = &mut self._regs;

        let res = regs.sp;
        regs.sp += Self::size_of_slots(2);        

        unsafe { *(res as *const c_void as *const T) }
    }
}
