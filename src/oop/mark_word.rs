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
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use std::sync::atomic::{AtomicU64, Ordering};

use crate::{utils::global_defs::uintx, OneBit};

#[repr(C)]
pub struct MarkWord {
    _encoded: AtomicU64,
}

impl MarkWord {
    pub fn new() -> Self {
        Self {
            _encoded: AtomicU64::new(0),
        }
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
#[cfg(target_pointer_width = "32")]
const HASH_BITS      :i32 = 25;
#[cfg(target_pointer_width = "64")]
const HASH_BITS      :i32 = 31;
#[cfg(target_pointer_width = "32")]
const KLASS_PTR_BITS :i32 = 32;
#[cfg(target_pointer_width = "64")]
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

const BIASED_TAG_MASK_IN_PLACE :u64 = BIASED_TAG_MASK << BIASED_TAG_SHIFT;
const AGE_MASK_IN_PLACE        :u64 = AGE_MASK        << AGE_SHIFT;

impl MarkWord {
    pub fn age(encoded: u64) -> i32 {
        ((encoded & AGE_MASK) >> AGE_SHIFT) as _
    }

    pub fn set_age(n: u64, age: i32) -> u64 {
        (n & !AGE_MASK_IN_PLACE) |  ((age as u64) << AGE_SHIFT)
    }

    pub fn inc_age(n: u64) -> u64 {
        let age = Self::age(n);
        debug_assert!(age != 15);

        Self::set_age(n, age + 1)
    }
}

