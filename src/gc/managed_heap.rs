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

use std::{cell::LazyCell, sync::{LazyLock, OnceLock}};

use crate::{memory::{bumper::Bumper, mem_region::MemRegion, virt_space::VirtSpace}, utils::global_defs::HeapWord};

static MANAGED_HEAP: OnceLock<ManagedHeap> = OnceLock::new();

#[derive(Debug)]
pub struct ManagedHeap {
    _virt_space: VirtSpace,
    _bumper: Bumper
}

unsafe impl Send for ManagedHeap {}
unsafe impl Sync for ManagedHeap {}

impl ManagedHeap {
    pub fn initialize(word_size: usize) {
        MANAGED_HEAP.set(Self::new(word_size)).unwrap()
    }

    fn new(word_size: usize) -> Self {
        let mut vm = VirtSpace::new(word_size, VirtSpace::page_byte_size(), false);
        vm.expand_by(word_size);
        
        let committed = vm.committed();

        Self {
            _virt_space: vm,
            _bumper: Bumper::new(committed)
        }
    }
}

impl ManagedHeap {
    pub fn heap() -> &'static ManagedHeap {
        MANAGED_HEAP.get().unwrap()
    }

    pub fn description() -> &'static str {
        "Do-nothing GC"
    }

    pub fn mem_region(&self) -> &MemRegion {
        self._virt_space.reserved()
    }

    pub fn mem_allocation(&self, word_size: usize, do_zero: bool) -> *const HeapWord {
        let res = self._bumper.par_alloc_with_size(word_size);
        assert!(!res.is_null(), "out of memory(managed heap).");

        if do_zero {
            unsafe {
                MemRegion::with_size(res as *const _, word_size).memset(0);
            }
        }

        res
    }
}
