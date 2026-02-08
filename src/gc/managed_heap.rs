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


use crate::{memory::{bumper::Bumper, mem_region::MemRegion, virt_space::VirtSpace}, utils::global_defs::{ByteSize, HeapWord, WordSize}};

#[derive(Debug)]
pub struct ManagedHeap {
    _virt_space: VirtSpace,
    _bumper: Bumper
}

unsafe impl Send for ManagedHeap {}
unsafe impl Sync for ManagedHeap {}

impl ManagedHeap {
    pub fn new(size: ByteSize) -> Self {
        let mut vm = VirtSpace::new(size, false);
        vm.expand_by(vm.reserved().size.into());
        
        let committed = vm.committed();

        Self {
            _virt_space: vm,
            _bumper: Bumper::new(committed)
        }
    }
}

impl ManagedHeap {
    pub fn description() -> &'static str {
        "Do-nothing GC"
    }

    pub fn mem_region(&self) -> &MemRegion {
        self._virt_space.reserved()
    }

    pub fn mem_allocation(&self, word_size: usize, do_zero: bool) -> *const HeapWord {
        unimplemented!()
    }
}
