use std::cell::RefCell;
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
use std::ops::{Deref, DerefMut};
use std::ptr::{self, null_mut};
use std::sync::Mutex;
use crate::common::universe;
use crate::memory::bump_alloc::BumpAllocator;
use crate::memory::mem_region::MemRegion;
use crate::memory::virt_space::VirtSpace;
use crate::OneBit;
use crate::oops::klass::{ArrayKlass, Klass, NormalKlass, PrimitiveKlass};
use crate::runtime::tls;
use crate::utils::global_defs::{address, naddr, word_t, K, LOG_BYTES_PER_ARCH};

const KLASS_MEM_SPACE_SIZE: usize = OneBit!() << (26 + LOG_BYTES_PER_ARCH);
const KLASS_MEM_BUCKET_SIZE: usize = 16 * K;

pub fn alloc_klass(klass: Klass) -> &'static Klass {
    let slot = tls::klass_mem_pool().alloc();
    unsafe {
        ptr::write(slot._data.deref_mut(), klass);
        &slot._data
    }
}

pub struct KlassMemSpace {
    _vm: Mutex<VirtSpace>,
    _base: address
}

impl KlassMemSpace {
    pub fn new() -> Self {
        let vm = VirtSpace::new(0, KLASS_MEM_SPACE_SIZE, 0, None, 0, false, false);
        let base = vm.mr().begin();

        Self {
            _vm: Mutex::new(vm),
            _base: base
        }
    }
}

unsafe impl Send for KlassMemSpace {}
unsafe impl Sync for KlassMemSpace {}

impl KlassMemSpace {
    pub fn compress(&self, raw: &Klass) -> naddr {
        let addr = raw as *const _ as address;
        (((addr - self._base) >> LOG_BYTES_PER_ARCH) + size_of::<word_t>()) as _
    }

    pub fn reslove(&self, comp: naddr) -> &Klass {
        let addr = self._base + ((comp as address - size_of::<word_t>()) << LOG_BYTES_PER_ARCH);
        unsafe { *(addr as *const _) }
    }
}

impl KlassMemSpace {
    pub fn new_bucket(&self) -> MemRegion {
        let mut guard = self._vm.lock().unwrap();

        let begin = guard.committed_region().end();
        let size = guard.expand_by(KLASS_MEM_BUCKET_SIZE, false);
        assert!(size != 0, "out of space");

        MemRegion::with_size(begin, size)
    }
}

union KlassMemSpaceSlot {
    _data: ManuallyDrop<Klass<'static>>,

    // Class unloading is unsupported currently.
    _dummy: word_t // _next: *mut Self,
}


impl KlassMemSpaceSlot {
    fn new() -> Self {
        Self { _dummy: 0 }
    }
}

#[derive(Debug)]
pub struct KlassMemPool {
    _bumper: RefCell<BumpAllocator>,
}

impl KlassMemPool {
    pub fn new() -> Self {
        Self { _bumper: RefCell::new(BumpAllocator::new()) }
    }
}

impl KlassMemPool {
    fn alloc(&self) -> &mut KlassMemSpaceSlot {
        let mut bumper = self._bumper.borrow_mut();

        let mut mem = bumper.alloc(size_of::<KlassMemSpaceSlot>());
        if mem == 0 {
            bumper.clear();

            bumper.init_with_mr(universe::klass_mem_space().new_bucket());
            mem = bumper.alloc(size_of::<KlassMemSpaceSlot>());

            assert!(mem != 0, "invariant");
        }

        unsafe { &mut *(mem as *mut KlassMemSpaceSlot) }
    }
}
