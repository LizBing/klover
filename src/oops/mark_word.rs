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

use modular_bitfield::{bitfield, prelude::{B2, B4, B26, B32}};

#[bitfield(bits = 32)]
#[derive(Clone, Copy)]
pub struct MarkWord {
    lock: B2,
    age: B4,
    hash: B26,
}

pub const NO_LOCK_VALUE: u8 = 0b00;
pub const LW_LOCK_VALUE: u8 = 0b01;
pub const HW_LOCK_VALUE: u8 = 0b10;
pub const GC_LOCK_VALUE: u8 = 0b11;

impl MarkWord {
    pub fn prototype() -> MarkWord {
        Self::new()
            .with_lock(NO_LOCK_VALUE)
            .with_age(0)
            .with_hash(0)
    }
}
