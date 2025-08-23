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

use std::{ptr::null_mut};
use std::cell::{Cell};
use once_cell::unsync::OnceCell;

use crate::engine::engine_globals::INTP_STACK_SIZE;
use crate::{align_up, memory::{allocation::c_heap_alloc, mem_region::MemRegion}, utils::global_defs::{addr_cast, address, word_t}};

pub type slot_t = usize;

struct Frame<T: Clone> {
    stored_bp: address,
    stored_data: T,
}

impl<T: Clone> Frame<T> {
    fn store(&mut self, bp: address, data: T) {
        self.stored_bp = bp;
        self.stored_data = data;
    }
}

#[derive(Debug)]
pub struct Context {
    _bp: Cell<address>,
    _sp: Cell<*mut slot_t>,

    _stack: once_cell::unsync::OnceCell<MemRegion>,
    _depth: Cell<usize>
}

impl Context {
    pub fn new() -> Self {
        Self {
            _sp: Cell::new(null_mut()),
            _bp: Cell::new(0),
            _stack: OnceCell::new(),
            _depth: Cell::new(0)
        }
    }

    pub fn init(&self) {
        let stack_mr = c_heap_alloc(INTP_STACK_SIZE.get_value()).expect("out of memory");
        self._stack.set(stack_mr.clone()).unwrap();
        self._sp.set(stack_mr.end() as _);
        self._bp.set(stack_mr.end());
    }
}

impl Context {
    pub const fn size_of_slots(n: usize) -> usize {
        size_of::<slot_t>() * n
    }

    pub const fn size_in_slots<T>() -> usize {
        align_up!(size_of::<T>(), size_of::<slot_t>()) / size_of::<slot_t>()
    }
}

impl Context {
    pub fn depth(&self) -> usize {
        self._depth.get()
    }

    fn stack(&self) -> &MemRegion {
        self._stack.get().unwrap()
    }
}

impl Context {
    // Ensure reachable.
    pub fn reserve(&self, slots: usize) -> bool {
        unsafe { self.stack().contains(self._sp.get().sub(slots) as _) }
    }

    pub fn alloca(&self, slots: usize, zeroing: bool) -> address {
        if !self.reserve(slots) {
            return 0;
        }
        unsafe { self._sp.set(self._sp.get().sub(slots)); }
        let res = self._sp.get() as _;

        if zeroing {
            MemRegion::with_size(res, Self::size_of_slots(slots)).memset(0);
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
    pub fn create_frame<T: Clone>(&self, data: T) -> bool {
        let mem = self.alloca(Self::size_in_slots::<Frame<T>>(), false);
        let new_frame;
        match addr_cast::<Frame<T>>(mem) {
            None => { return false; },
            Some(n) => new_frame = n,
        }

        new_frame.store(self._bp.get(), data);
        self._bp.set(mem);

        self._depth.set(self.depth() + 1);

        true
    }

    // helper
    fn cal_unwind_sp<T: Clone>(bp: address) -> *mut slot_t {
        (bp + Self::size_in_slots::<Frame<T>>()) as _
    }

    pub fn unwind<T: Clone>(&self) -> Option<T> {
        if self.depth() == 0 {
            return None;
        }
        let frame = addr_cast::<Frame<T>>(self._bp.get()).unwrap();

        self._sp.set(Self::cal_unwind_sp::<T>(self._bp.get()));
        self._bp.set(frame.stored_bp);

        self._depth.set(self.depth() - 1);

        Some(frame.stored_data.clone())
    }

    // Unsafe unless reserved.
    pub fn push<const SLOTS: usize, T>(&self, n: T) {
        unsafe {
            self._sp.set(self._sp.get().sub(SLOTS));

            *(self._sp.get() as address as *mut T) = n;
        }
    }

    pub fn pop<const SLOTS: usize, T: Copy>(&self) -> T {
        let addr = self._sp.get();
        unsafe {
            self._sp.set(addr.add(SLOTS));
            *(addr as address as *const T)
        }
    }
}
