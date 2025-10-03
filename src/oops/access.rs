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

use std::{marker::PhantomData, ptr::null_mut, sync::atomic::{AtomicI32, AtomicI64, AtomicI8, AtomicUsize, Ordering}};

use crate::{common::universe, gc::common::barrier_set::AccessBarriers, oops::oop::{self, as_oop, ObjPtr}, utils::global_defs::{naddr, word_t, LOG_BYTES_PER_ARCH}, OneBit};
use crate::utils::global_defs::address;

bitflags::bitflags! {
    pub struct DecoratorSet: u32 {
        const DECORATOR_NONE = 0;

        // const AS_RAW = OneBit!() << 1;
        // const AS_OOP = OneBit!() << 2;
        // const COMPRESSED = OneBit!() << 3;

        const MO_UNORDERED      = OneBit!() << 6;
        const MO_RELAXED        = OneBit!() << 7;
        const MO_ACQUIRE        = OneBit!() << 8;
        const MO_RELEASE        = OneBit!() << 9;
        const MO_SEQ_CST        = OneBit!() << 10;

        const IN_HEAP = OneBit!() << 11;

        const ACCESS_READ = OneBit!() << 20;
        const ACCESS_WRITE = OneBit!() << 21;
    }
}

/*
#[inline]
pub fn encode_coop(addr: address) -> naddr {
    if addr == 0 { return 0; }

    let base = universe::coops_base();
    ((addr - base - size_of::<word_t>()) >> LOG_BYTES_PER_ARCH) as _
}

#[inline]
pub fn decode_coop(addr: naddr) -> address {
    if addr == 0 { return 0;}
    
    let base = universe::coops_base();
    ((addr as address) << LOG_BYTES_PER_ARCH) + base + size_of::<word_t>()
}
*/

pub struct AccessOps<const D: u32>;
impl<const D: u32> AccessOps<D> {
    pub const fn flags() -> DecoratorSet {
        DecoratorSet::from_bits_truncate(D)
    }

    pub const fn cast_mem_order() -> Option<Ordering> {
        if Self::flags().contains(DecoratorSet::MO_RELAXED) {
            Some(Ordering::Relaxed)
        } else if Self::flags().contains(DecoratorSet::MO_ACQUIRE) {
            Some(Ordering::Acquire)
        } else if Self::flags().contains(DecoratorSet::MO_RELAXED) {
            Some(Ordering::Release)
        } else if Self::flags().contains(DecoratorSet::MO_SEQ_CST) {
            Some(Ordering::SeqCst)
        } else {
            None
        }
    }

    #[inline]
    pub fn load<T: Copy>(addr: address) -> T {
        
    }

    #[inline]
    pub fn store<T>(addr: address, value: T) {
    }
}

pub struct Access<Barrier: AccessBarriers<D>, const D: u32>(PhantomData<Barrier>);
impl<Barrier: AccessBarriers<D>, const D: u32> Access<Barrier, D> {
    const fn flags() -> DecoratorSet {
        DecoratorSet::from_bits_truncate(D)
    }

    #[inline]
    pub fn load_at<T: Copy>(oop: ObjPtr, offs: usize) -> T {
        if Self::flags().contains(DecoratorSet::IN_HEAP) {
            Barrier::load_in_heap_at(oop, offs)
        } else {
            let addr = oop as address + offs;
            AccessOps::<D>::load(addr)
        }
    }

    #[inline]
    pub fn oop_load_at(oop: ObjPtr, offs: usize) -> ObjPtr {
        Barrier::oop_load_in_heap_at(oop, offs)
    }

    #[inline]
    pub fn store_at<T: Copy>(oop: ObjPtr, offs: usize, value: T) {
        Barrier::store_in_heap_at(oop, offs, value);
    }

    #[inline]
    pub fn oop_store_at(oop: ObjPtr, offs: usize, value: ObjPtr) {
        Barrier::oop_store_in_heap_at(oop, offs, value);
    }

    #[inline]
    pub fn oop_cas_in_heap_at(oop: ObjPtr, offs: usize, exp: ObjPtr, des: ObjPtr) -> bool {
        Barrier::oop_cas_in_heap_at(oop, offs, exp, des)
    }

    #[inline]
    pub fn oop_xchg_in_heap_at(oop: ObjPtr, offs: usize, new_value: ObjPtr) -> ObjPtr {
        Barrier::oop_xchg_in_heap_at(oop, offs, new_value)
    }

    #[inline]
    pub fn array_copy(dst: ObjPtr, dst_offs: usize,
                              src: ObjPtr, src_offs: usize,
                              elem: usize, length: usize)
    {
        Barrier::array_copy_in_heap(dst, dst_offs, src, src_offs, elem, length);
    }

    #[inline]
    pub fn oop_array_copy(dst: ObjPtr, dst_offs: usize,
                                  src: ObjPtr, src_offs: usize,
                                  length: usize)
    {
        Barrier::oop_array_copy_in_heap(dst, dst_offs, src, src_offs, length);
    }

    #[inline]
    pub fn clone_in_heap(dst: ObjPtr, src: ObjPtr, size: usize) {
        Barrier::clone_in_heap(dst, src, size);
    }
}
