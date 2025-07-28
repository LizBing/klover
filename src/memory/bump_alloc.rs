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

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::utils::global_defs::address;

use super::mem_region::MemRegion;

pub struct BumpAllocator {
    _mr: MemRegion,
    _top: AtomicUsize
}

impl BumpAllocator {
    pub fn new() -> Self {
        Self {
            _mr: MemRegion::new(),
            _top: AtomicUsize::new(0)
        } 
    }

    pub fn with_mr(mr: MemRegion) -> Self {
        let mut res = Self::new();
        res.init_with_mr(mr);

        res
    }
}

impl BumpAllocator {
    pub fn init_with_mr(&mut self, mr: MemRegion) {
        self._mr = mr; 
        self.clear();
    }
}

impl BumpAllocator {
    pub fn size(&self) -> usize {
        self._mr.size()
    }

    pub fn remaining(&self) -> usize {
        (unsafe { *self._top.as_ptr() }) - self._mr.begin()
    }

    pub fn mr(&self) -> MemRegion {
        self._mr
    }
}

impl BumpAllocator {
    pub fn clear(&mut self) {
        *self._top.get_mut() = self._mr.begin()
    }
}

impl BumpAllocator {
    pub fn alloc(&mut self, size: usize) -> address {
        let top = self._top.get_mut();

        let new_top = *top + size;
        if new_top > self._mr.end() { return 0; }

        let res = *top;
        *top = new_top;

        res
    }

    pub fn par_alloc(&self, size: usize) -> address {
        let mut res = self._top.load(Ordering::SeqCst);

        loop {
            let new_top = res + size;

            if new_top > self._mr.end() { return 0; }

            match self._top.compare_exchange_weak(res, new_top, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => break,
                Err(x) => res = x
            }
        }

        res
    }
}

impl BumpAllocator {
    pub fn expand_by(&mut self, size: usize) {
        let new_end = self._mr.end() + size;
        self._mr.set_end(new_end);
    }

    pub fn expand_and_par_alloc(&mut self, exp_size: usize, alloc_size: usize) -> address {
        let mut res = self._top.load(Ordering::SeqCst);

        loop {
            let new_top = res + alloc_size;

            match self._top.compare_exchange_weak(res, new_top, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => break,
                Err(x) => res = x
            }
        }

        self.expand_by(exp_size);

        res
    }
}

