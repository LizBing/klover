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

use crate::{common::universe, gc::common::barrier_set::AccessBarriers, oops::oop::{as_oop, ObjPtr}, utils::global_defs::{naddr, word_t, LOG_BYTES_PER_ARCH}, OneBit};
use crate::utils::global_defs::address;

pub struct RawAccess;
impl RawAccess {
    #[inline]
    pub unsafe fn load_raw<T: Copy>(addr: address) -> T {
        *(addr as *const _)
    }

    #[inline]
    pub unsafe fn store_raw<T>(addr: address, value: T) {
        *(addr as *mut _) = value;
    }
}

pub struct MemoryAccess;
impl MemoryAccess {
    #[inline]
    pub fn load_volatile<T: Copy>(addr: address) -> T {
        unsafe { (addr as *const T).read_volatile() }
    }

    #[inline]
    pub fn store_volatile<T>(addr: address, value: T) {
        unsafe { (addr as *mut T).write_volatile(value); }
    }

    #[inline]
    pub fn cas_int(addr: address, exp: i32, des: i32) -> bool {
        unsafe { AtomicI32::from_ptr(addr as _).compare_exchange(exp, des, Ordering::SeqCst, Ordering::SeqCst).is_ok() }
    }

    #[inline]
    pub fn cas_int_weak(addr: address, exp: i32, des: i32) -> bool {
        unsafe { AtomicI32::from_ptr(addr as _).compare_exchange_weak(exp, des, Ordering::SeqCst, Ordering::SeqCst).is_ok() }
    }

    #[inline]
    pub fn cas_long(addr: address, exp: i64, des: i64) -> bool {
        unsafe { AtomicI64::from_ptr(addr as _).compare_exchange(exp, des, Ordering::SeqCst, Ordering::SeqCst).is_ok() }
    }

    #[inline]
    pub fn cas_long_weak(addr: address, exp: i64, des: i64) -> bool {
        unsafe { AtomicI64::from_ptr(addr as _).compare_exchange_weak(exp, des, Ordering::SeqCst, Ordering::SeqCst).is_ok() }
    }
    
    #[inline]
    pub fn cas_ptr(addr: address, exp: address, des: address) -> bool {
        unsafe { AtomicUsize::from_ptr(addr as _).compare_exchange(exp, des, Ordering::SeqCst, Ordering::SeqCst).is_ok() }
    }

    #[inline]
    pub fn cas_ptr_weak(addr: address, exp: address, des: address) -> bool {
        unsafe { AtomicUsize::from_ptr(addr as _).compare_exchange_weak(exp, des, Ordering::SeqCst, Ordering::SeqCst).is_ok() }
    }
}

bitflags::bitflags! {
    pub struct DecoratorSet: u32 {
        const DECORATOR_NONE = 0;

        // const AS_RAW = OneBit!() << 1;
        // const AS_OOP = OneBit!() << 2;
        const VOLATILE = OneBit!() << 3;
        const COMPRESSED = OneBit!() << 4;
        const CAS = OneBit!() << 5;
    }
}

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

trait AccessOps<Barriers: AccessBarriers<D>, const D: u32> {
    #[inline]
    fn flags() -> DecoratorSet {
        DecoratorSet::from_bits_truncate(D)
    }

    #[inline]
    fn load<T: Copy>(addr: address) -> T {
        if Self::flags().contains(DecoratorSet::VOLATILE) {
            MemoryAccess::load_volatile(addr)
        } else {
            unsafe { RawAccess::load_raw(addr) }
        }
    }

    #[inline]
    fn store<T>(addr: address, value: T) {
        if Self::flags().contains(DecoratorSet::VOLATILE) {
            MemoryAccess::store_volatile(addr, value);
        } else {
            unsafe { RawAccess::store_raw(addr, value); }
        }
    }

    #[inline]
    fn do_slot<Ret, F>(slot_addr: address, f: F) -> Ret
    where
        Ret: Copy,
        F: Fn(&mut address) -> Ret
    {
        let res;
        let mut slot;
        if Self::flags().contains(DecoratorSet::COMPRESSED) {
            unsafe { slot = decode_coop(RawAccess::load_raw(slot_addr)); }
            let tmp = slot;

            res = f(&mut slot);

            if tmp != slot {
                if Self::flags().contains(DecoratorSet::CAS) {
                    MemoryAccess::cas_int(slot_addr, encode_coop(tmp) as _, encode_coop(slot) as _);
                } else {
                    unsafe { RawAccess::store_raw(slot_addr, encode_coop(slot)); }
                }
            }
        } else {
            unsafe { slot = RawAccess::load_raw(slot_addr); }
            let tmp = slot;

            res = f(&mut slot);

            if tmp != slot {
                if Self::flags().contains(DecoratorSet::CAS) {
                    MemoryAccess::cas_ptr(slot_addr, tmp, slot);
                } else {
                    unsafe { RawAccess::store_raw(slot_addr, slot); }
                }
            }
        }

        res
    }
}

pub trait HeapAccess<Barriers: AccessBarriers<D>, const D: u32> {
    type Ops: AccessOps<Barriers, D>;

    #[inline]
    fn load_at<T: Copy>(slot_addr: address, offs: usize) -> T {
        Self::Ops::do_slot(slot_addr, |slot| {
            let 8
        })
    }

    #[inline]
    fn oop_load_at(slot_addr: address, offs: usize) -> address {
        Self::Ops::do_slot(slot_addr, |slot| {
            Barriers::oop_load_at(*slot, offs)
        })
    }

    #[inline]
    fn store_at<T: Copy>(slot_addr: address, offs: usize, value: T) {
        Self::Ops::do_slot(slot_addr, |slot| {
            Barriers::store_at(*slot, offs, value);
        })
    }

    #[inline]
    fn oop_store_at(slot_addr: address, offs: usize, value: address) {
        Self::Ops::do_slot(slot_addr, |slot| {
            Barriers::oop_store_at(*slot, offs, value);
        })
    }
}

pub trait InterpreterAccess<Barriers: AccessBarriers<D>, const D: u32> {
    type Ops: AccessOps<Barriers, D>;

    #[inline]
    fn oop_load(slot_addr: address) -> address {
        Self::Ops::do_slot(slot_addr, |slot| {
            Barriers::oop_load(slot)
        })
    }

    #[inline]
    fn oop_store(slot_addr: address, value: address) {
        Self::Ops::do_slot(slot_addr, |slot| {
            Barriers::oop_store(slot, value);
        })
    }
}
