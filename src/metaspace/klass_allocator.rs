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
use std::mem::ManuallyDrop;
use std::sync::Mutex;
use crate::common::universe;
use crate::memory::bump_alloc::BumpAllocator;
use crate::memory::mem_region::MemRegion;
use crate::memory::virt_space::VirtSpace;
use crate::metaspace::klass_cell::KlassCell;
use crate::OneBit;
use crate::oops::klass::{Klass};
use crate::runtime::tls;
use crate::utils::global_defs::{address, naddr, word_t, K, LOG_BYTES_PER_ARCH};

const KLASS_MEM_SPACE_SIZE: usize = OneBit!() << (26 + LOG_BYTES_PER_ARCH);
const KLASS_MEM_BUCKET_SIZE: usize = 16 * K;

pub fn alloc_klass() -> KlassCell {
    KlassCell::with_raw(tls::klass_mem_pool().alloc())
}

pub struct KlassMemSpace {
    _mtx: Mutex<()>,
    _vm: VirtSpace,
    _base: address
}

impl KlassMemSpace {
    pub fn new() -> Self {
        let vm = VirtSpace::new(0, KLASS_MEM_SPACE_SIZE, 0, false, false).unwrap();
        let base = vm.mr().begin();

        Self {
            _mtx: Mutex::new(()),
            _vm: vm,
            _base: base
        }
    }
}

impl KlassMemSpace {
    pub fn compress(&self, raw: *mut Klass) -> naddr {
        let addr = raw as address;
        (((addr - self._base) >> LOG_BYTES_PER_ARCH) + size_of::<word_t>()) as _
    }

    pub fn reslove(&self, comp: naddr) -> KlassCell {
        let addr = self._base + ((comp as address - size_of::<word_t>()) << LOG_BYTES_PER_ARCH);
        KlassCell::with_raw(addr as _)
    }
}

impl KlassMemSpace {
    pub fn new_bucket(&mut self) -> MemRegion {
        let guard = self._mtx.lock().unwrap();

        let begin = self._vm.committed_region().end();
        let size = self._vm.expand_by(KLASS_MEM_BUCKET_SIZE, false);
        assert!(size != 0, "out of space");

        MemRegion::with_size(begin, size)
    }
}

union KlassMemSpaceSlot {
    // Class unloading is unsupported currently.
    // _next: *mut Self,
    _data: ManuallyDrop<Klass<'static>>,
    _dummy: word_t
}


impl KlassMemSpaceSlot {
    fn new() -> Self {
        Self { _dummy: 0 }
    }
}

#[derive(Debug)]
pub struct KlassMemPool {
    _bumper: BumpAllocator,
}

impl KlassMemPool {
    pub fn new() -> Self {
        Self { _bumper: BumpAllocator::new() }
    }
}

impl KlassMemPool {
    fn alloc(&mut self) -> *mut Klass {
        let bumper = &mut self._bumper;

        let mut res = bumper.alloc(size_of::<KlassMemSpaceSlot>());
        if res == 0 {
            bumper.init_with_mr(universe::klass_mem_space().new_bucket());
            res = bumper.alloc(size_of::<KlassMemSpaceSlot>());
        }

        res as _
    }
}
