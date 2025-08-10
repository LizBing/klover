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
use std::cell::{Cell, UnsafeCell};
use crate::engine::engine_globals::INTP_STACK_SIZE;
use crate::{align_up, memory::{allocation::c_heap_alloc, mem_region::MemRegion, virt_space::VirtSpace}, utils::global_defs::{addr_cast, address, word_t}};

pub type slot_t = usize;

struct Frame<T> {
    stored_bp: address,
    stored_data: T,
}

impl<T> Frame<T> {
    fn store(&mut self, bp: address, data: T) {
        self.stored_bp = bp;
        self.stored_data = data;
    }
}

pub struct VMRegiters {
    sp: address,
    bp: address
}

impl VMRegiters {
    fn new() -> Self {
        VMRegiters { sp: 0, bp: 0 }
    }
}

#[derive(Debug)]
pub struct Context {
    _regs: UnsafeCell<VMRegiters>,
    _stack: MemRegion,
    _depth: Cell<usize>
}

impl Context {
    pub fn new() -> Self {
        Self {
            _regs: UnsafeCell::new(VMRegiters::new()),
            _stack: c_heap_alloc(align_up!(INTP_STACK_SIZE.get_value(), VirtSpace::page_size())).unwrap(),
            _depth: Cell::new(0)
        }
    }
}

impl Context {
    pub const fn size_of_slots(n: usize) -> usize {
        size_of::<slot_t>() * n
    }
}

impl Context {
    fn get_regs(&self) -> &'_ mut VMRegiters {
        unsafe { &mut *self._regs.get() }
    }

    pub fn depth(&self) -> usize {
        self._depth.get()
    }
}

impl Context {
    // Ensure reachable.
    pub fn reserve(&self, slots: usize) -> bool {
        let size = Self::size_of_slots(slots);
        let regs = self.get_regs();

        self._stack.contains(regs.sp - size)
    }

    pub fn alloca(&self, slots: usize, zeroing: bool) -> address {
        let size = Self::size_of_slots(slots);
        let regs = self.get_regs();
        
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
/*
    pub fn try_free(&self, addr: address, slots: usize) -> bool {
        let size = Self::size_of_slots(slots);
        
        let regs = self.get_regs();
        if regs.sp != addr && !self._stack.contains(regs.sp + size) { return false; }

        regs.sp += size;

        true
    }
*/
    pub fn create_frame<T>(&self, data: T) -> bool {
        let regs = self.get_regs();

        let mem = self.alloca(size_of::<Frame<T>>(), false);
        let new_frame;
        match addr_cast::<Frame<T>>(mem) {
            None => { return false; },
            Some(n) => new_frame = n,
        }

        new_frame.store(regs.bp, data);
        regs.bp = new_frame as *const _ as _;

        self._depth.set(self.depth() + 1);

        true
    }

    // helper
    fn cal_unwind_sp<T>(bp: address) -> address {
        bp + size_of::<Frame<T>>()
    }

    pub fn unwind<T: Copy>(&self) -> Option<T> {
        if self.depth() == 0 {
            return None;
        }

        let regs = self.get_regs();

        let frame;
        match addr_cast::<Frame<T>>(regs.bp) {
            Some(n) => frame = n,
            _ => unreachable!()
        }

        regs.sp = Self::cal_unwind_sp::<T>(regs.bp);
        regs.bp = frame.stored_bp;

        self._depth.set(self.depth() - 1);

        Some(frame.stored_data)
    }

    // Unsafe unless reserved.
    pub fn push<const SLOTS: usize, T: Copy>(&self, n: T) {
        let regs = self.get_regs();
        regs.sp -= Self::size_of_slots(SLOTS);

        unsafe { *(regs.sp as *mut c_void as *mut T) = n; }
    }

    pub fn pop<const SLOTS: usize, T: Copy>(&self) -> T {
        let regs = self.get_regs();

        let res = regs.sp;
        regs.sp += Self::size_of_slots(SLOTS);

        unsafe { *(res as *const c_void as *const T) }
    }
}
