/*
 * Copyright (c) 2025, Lei Zaakjyu. All rights reserved.
 *
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS
 * OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use crate::{util::global_defs::{uintx, BITS_PER_ARCH}, OneBit};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Copy, Clone)]
pub enum LockType {
    Locked      = 0b00,
    Unlocked    = 0b01,
    Monitor     = 0b10,
    Marked      = 0b11
}

// Since the klass ptr would be made only 28 bits,
// we have to firstly compress it down to 32 bits,
// keep its alignment as 128(2^3 * 2^4) bytes.

const LOCK_BITS:            i32 = 2;
const SELF_FWDED_TAG_BIT:   i32 = 1;
const AGE_BITS:             i32 = 4;
const HASH_BITS:            i32 = 29;
const KLASS_PTR_BITS:       i32 = BITS_PER_ARCH -
                                  LOCK_BITS -
                                  SELF_FWDED_TAG_BIT -
                                  AGE_BITS -
                                  HASH_BITS;

const LOCK_SHIFT:           i32 = 0;
const SELF_FWDED_TAG_SHIFT: i32 = LOCK_SHIFT + LOCK_BITS;
const AGE_SHIFT:            i32 = SELF_FWDED_TAG_SHIFT + SELF_FWDED_TAG_BIT;
const HASH_SHIFT:           i32 = AGE_SHIFT + AGE_BITS;
const KLASS_PTR_SHIFT:      i32 = HASH_SHIFT + HASH_BITS;

const LOCK_MASK:            uintx = (OneBit!() << LOCK_BITS)          - 1;
const SELF_FWDED_TAG_MASK:  uintx = (OneBit!() << SELF_FWDED_TAG_BIT) - 1;
const AGE_MASK:             uintx = (OneBit!() << AGE_BITS)           - 1;
const HASH_MASK:            uintx = (OneBit!() << HASH_BITS)          - 1;
const KLASS_PTR_MASK:       uintx = (OneBit!() << KLASS_PTR_BITS)     - 1;

const LOCK_MASK_IN_PLACE:           uintx = LOCK_MASK          << LOCK_SHIFT;
const SELF_FWDED_TAG_MASK_IN_PLACE: uintx = SELF_FWDED_TAG_MASK << SELF_FWDED_TAG_SHIFT;
const AGE_MASK_IN_PLACE:            uintx = AGE_MASK           << AGE_SHIFT;
const HASH_MASK_IN_PLACE:           uintx = HASH_MASK          << HASH_SHIFT;
const KLASS_PTR_MASK_IN_PLACE:      uintx = KLASS_PTR_MASK    << KLASS_PTR_SHIFT;

#[repr(C)]
pub struct MarkWord {
    _value: AtomicUsize
}

impl MarkWord {
    pub fn with_arch(n: uintx) -> Self {
        MarkWord { _value: AtomicUsize::new(n as _) }
    }

    pub fn with_ref<T>(n: &T) -> Self {
        MarkWord { _value: AtomicUsize::new(n as *const T as _) }
    }
}

impl MarkWord {
    pub fn with_lock_raw(ptr: uintx, t: LockType) -> uintx {
        ptr | t as uintx
    }

    pub fn with_fwd_ptr_raw(ptr: uintx) -> uintx {
        ptr | (OneBit!() << SELF_FWDED_TAG_SHIFT) | LockType::Marked as uintx
    }
}

impl MarkWord {
    fn raw(&self) -> uintx {
        unsafe { *self._value.as_ptr() }
    }

    fn lock_type(&self) -> LockType {
        match (self.raw() >> LOCK_SHIFT) & LOCK_MASK {
            0 => LockType::Locked,
            1 => LockType::Unlocked,
            2 => LockType::Monitor,
            3 => LockType::Marked,

            _ => unreachable!()
        }
    }

    fn is_self_fwded(&self) -> bool {
        self.raw() & SELF_FWDED_TAG_MASK_IN_PLACE != 0
    }

    fn age(&self) -> i32 {
        ((self.raw() >> AGE_SHIFT) & AGE_MASK) as _
    }

    fn hash(&self) -> i32 {
        ((self.raw() >> HASH_SHIFT) & HASH_MASK) as _
    }

    fn klass_ptr(&self) -> i32 {
        ((self.raw() >> KLASS_PTR_SHIFT) & KLASS_PTR_MASK) as _
    }

    /// Atomic compare-and-swap. Returns true if successful.
    pub fn atomic_cas(&self, old: uintx, new: uintx) -> Result<usize, usize> {
        self._value.compare_exchange(old, new , Ordering::SeqCst, Ordering::SeqCst)
    }

    /// Atomically update the value using a function in a CAS loop.
    fn atomic_update<F>(&self, f: F)
    where
        F: Fn(uintx) -> uintx,
    {
        let mut current = self._value.load(Ordering::SeqCst) as uintx;
        loop {
            let new = f(current);
            match self._value.compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => break,
                Err(actual) => current = actual
            }
        }
    }
}

impl MarkWord {
    fn age_inc(&self) {}
}
