/*
 * Copyright 2026 Lei Zaakjyu
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

use std::ptr::null;

use crate::{memory::virt_space::VirtSpace, utils::global_defs::{Address, G, LOG_BYTES_PER_WORD, Word}};

const SLOT_BYTE_SIZE: usize = size_of::<Word>();
const MAX_CAPACITY_IN_BYTE: usize = 32 * G - SLOT_BYTE_SIZE;

pub type NarrowPtr = u32;

// A wrapper for VirtSpace
#[derive(Debug)]
pub struct CompressedSpace {
    _vs: VirtSpace,
}

impl CompressedSpace {
    pub fn new(vs: VirtSpace) -> Self {
        assert!(vs.reserved().size_in_bytes() <= MAX_CAPACITY_IN_BYTE);

        Self {
            _vs: vs
        }
    }
}

impl CompressedSpace {
    pub fn null_narrow() -> NarrowPtr { 0 }

    pub fn vs(&self) -> &VirtSpace {
        &self._vs
    }
}

impl CompressedSpace {
    #[inline(always)]
    pub fn base(&self) -> Address {
        self._vs.reserved().start() as _
    } 

    #[inline(always)]
    pub fn encode<T>(&self, ptr: *const T) -> NarrowPtr {
        if ptr.is_null() { return Self::null_narrow(); }

        (((ptr as Address) - self.base() + SLOT_BYTE_SIZE) >> LOG_BYTES_PER_WORD) as _
    }

    #[inline(always)]
    pub fn decode<T>(&self, nptr: NarrowPtr) -> *const T {
        if nptr == Self::null_narrow() { return null(); }

        (((nptr as Address) << LOG_BYTES_PER_WORD) - SLOT_BYTE_SIZE + self.base()) as _
    }
}
