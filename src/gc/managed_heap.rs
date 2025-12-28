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

use std::{cell::LazyCell, sync::LazyLock};

use crate::{memory::{mem_region::MemRegion, virt_space::VirtSpace}, utils::global_defs::HeapWord};

static MANAGED_HEAP: LazyLock<ManagedHeap> = LazyLock::new(ManagedHeap::new);
pub struct ManagedHeap {
    _virt_space: VirtSpace,
}

unsafe impl Send for ManagedHeap {}
unsafe impl Sync for ManagedHeap {}

impl ManagedHeap {
    pub fn new() -> Self {
        unimplemented!()
    }
}

impl ManagedHeap {
    pub fn description() -> &'static str {
        "Do-nothing GC"
    }

    pub fn mem_region() -> &'static MemRegion {
        MANAGED_HEAP._virt_space.reserved()
    }

    pub fn mem_allocation(word_size: usize) -> *const HeapWord {
        unimplemented!()
    }

    pub fn allocation_for_tlab(word_size: usize, act_size: &usize) -> *const HeapWord {
        unimplemented!()
    }
}
