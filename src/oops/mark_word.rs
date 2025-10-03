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

use std::sync::atomic::{AtomicU64, Ordering};

use crate::common::universe;
use crate::oops::klass_handle::KlassHandle;
use crate::{utils::global_defs::uintx, OneBit};
use crate::utils::global_defs::naddr;
use super::mark_word;

#[repr(C)]
pub struct MarkWord {
    _encoded: AtomicU64,
    _klass: KlassHandle,
}

impl MarkWord {
    pub fn prototype(klass: KlassHandle) -> Self {
        unimplemented!()
    }
}

impl MarkWord {
    pub fn encoded(&self) -> u64 {
        self._encoded.load(Ordering::SeqCst)
    }

    pub fn set_encoded(&self, n: u64) {
        self._encoded.store(n, Ordering::SeqCst);
    }
}

const LOCK_BITS      :i32 =  2;
const BIASED_TAG_BIT :i32 =  1;
const AGE_BITS       :i32 =  4;
const HASH_BITS      :i32 = 31;
const KLASS_PTR_BITS :i32 = 26;

const LOCK_SHIFT       :i32 = 0;
const BIASED_TAG_SHIFT :i32 = LOCK_SHIFT       + LOCK_BITS;
const AGE_SHIFT        :i32 = BIASED_TAG_SHIFT + BIASED_TAG_BIT;
const HASH_SHIFT       :i32 = AGE_SHIFT        + AGE_SHIFT;
const KLASS_PTR_SHIFT  :i32 = HASH_SHIFT       + HASH_BITS;

const LOCK_MASK       :u64 = (OneBit!() << LOCK_BITS)      - 1;
const BIASED_TAG_MASK :u64 = (OneBit!() << BIASED_TAG_BIT) - 1;
const AGE_MASK        :u64 = (OneBit!() << AGE_BITS)       - 1;
const KLASS_PTR_MASK  :u64 = (OneBit!() << KLASS_PTR_BITS) - 1;
const HASH_MASK       :u64 = (OneBit!() << HASH_BITS)      - 1;

const LOCK_MASK_IN_PLACE       :u64 = LOCK_MASK << LOCK_SHIFT;
const BIASED_TAG_MASK_IN_PLACE :u64 = BIASED_TAG_MASK << BIASED_TAG_SHIFT;
const AGE_MASK_IN_PLACE        :u64 = AGE_MASK        << AGE_SHIFT;
const KLASS_PTR_MASK_IN_PLACE  :u64 = KLASS_PTR_MASK << KLASS_PTR_SHIFT;
const HASH_MASK_IN_PLACE        :u64 = HASH_MASK << HASH_SHIFT;

pub const LOCKED_VALUE   :u64 = 0b00;
pub const UNLOCKED_VALUE :u64 = 0b01;
pub const MONITORED_VALUE  :u64 = 0b10;
pub const MARKED_VALUE   :u64 = 0b11;


impl MarkWord {
    pub fn lock_value(encoded: u64) -> u64 {
        (encoded & LOCK_MASK) >> LOCK_SHIFT
    }

    pub fn set_lock(n: u64, lv: u64) -> u64 {
        (n & !LOCK_MASK_IN_PLACE) | (lv << LOCK_SHIFT)
    }
    
    pub fn is_biased(encoded: u64) -> bool {
        encoded & !BIASED_TAG_MASK_IN_PLACE != 0
    }
    
    pub fn set_biased_tag(n: u64) -> u64 {
        (n & !BIASED_TAG_MASK_IN_PLACE) | (OneBit!() << BIASED_TAG_SHIFT)
    }
    
    pub fn unset_biased_tag(n: u64) -> u64 {
        n & !BIASED_TAG_MASK_IN_PLACE
    }

    pub fn age(encoded: u64) -> i32 {
        ((encoded & AGE_MASK_IN_PLACE) >> AGE_SHIFT) as _
    }

    pub fn set_age(n: u64, age: i32) -> u64 {
        (n & !AGE_MASK_IN_PLACE) |  ((age as u64) << AGE_SHIFT)
    }
    
    pub fn klass_ptr(encoded: u64) -> naddr {
        ((encoded & KLASS_PTR_MASK_IN_PLACE) >> KLASS_PTR_SHIFT) as _
    }
    
    pub fn set_klass_ptr(n: u64, addr: naddr) -> u64 {
        (n & !KLASS_PTR_MASK_IN_PLACE) |  ((addr as u64) << KLASS_PTR_SHIFT)
    }
    
    pub fn hash(encoded: u64) -> i32 {
        ((encoded & HASH_MASK_IN_PLACE) >> HASH_SHIFT) as _
    }
    
    pub fn set_hash(n: u64, hash: i32) -> u64 {
        (n & !HASH_MASK_IN_PLACE) | ((hash as u64) << HASH_SHIFT)
    }
}

