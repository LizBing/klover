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

use modular_bitfield::{bitfield, prelude::{B1, B2, B26, B31, B4}};

#[bitfield(bits = 64)]
#[derive(Clone, Copy)]
pub struct MarkWord {
    lock: B2,
    biased: B1,
    age: B4,
    hash: B31,
    klass_comp_ptr: B26
}

pub const NO_LOCK_VALUE: u8 = 0b00;
pub const LW_LOCK_VALUE: u8 = 0b01;
pub const HW_LOCK_VALUE: u8 = 0b10;
pub const GC_LOCK_VALUE: u8 = 0b11;

pub const NO_BIASED: u8 = 0b00;
pub const BIASED_VALUE: u8 = 0b01;

impl MarkWord {
    pub fn prototype(klass_comp_ptr: u32) -> MarkWord {
        Self::new()
            .with_lock(NO_LOCK_VALUE)
            .with_biased(NO_BIASED)
            .with_age(0)
            .with_hash(0)
            .with_klass_comp_ptr(klass_comp_ptr)
    }
}

pub struct AtomicMarkWord {
    _value: AtomicU64
}

impl AtomicMarkWord {
    pub fn load(&self) -> MarkWord {
        MarkWord::from_bytes(self._value.load(Ordering::SeqCst).to_le_bytes())
    }

    pub fn store(&self, mw: MarkWord) {
        let raw = unsafe { std::mem::transmute::<MarkWord, u64>(mw) };
        self._value.store(raw, Ordering::SeqCst);
    }

    pub fn cmp_xchg(&self, exp: MarkWord, des: MarkWord)
        -> Result<MarkWord, MarkWord>
    {
        let exp_raw = unsafe { std::mem::transmute::<MarkWord, u64>(exp) };
        let des_raw = unsafe { std::mem::transmute::<MarkWord, u64>(des) };
        match self._value.compare_exchange(exp_raw, des_raw, Ordering::SeqCst, Ordering::Relaxed) {
            Ok(x) => Ok(unsafe { std::mem::transmute(x) }),
            Err(x) => Err(unsafe { std::mem::transmute(x) }),
        }
    }
}
