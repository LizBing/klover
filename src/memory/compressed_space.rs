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

use crate::{align_down, memory::virt_space::VirtSpace, utils::global_defs::{Address, ByteSize, G, HeapWord, LOG_BYTES_PER_WORD}};

const SLOT_SIZE: ByteSize = ByteSize(size_of::<HeapWord>());

pub type NarrowAddr = u32;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct NarrowPtr(NarrowAddr);

impl NarrowPtr {
    pub const fn null() -> Self {
        NarrowPtr(0)
    }

    pub const fn new(value: NarrowAddr) -> Self {
        Self(value)
    }

    pub const fn value(self) -> NarrowAddr {
        self.0
    }

    pub const fn is_null(self) -> bool {
        self.value() == 0
    }
}

// A wrapper for VirtSpace
#[derive(Debug)]
pub struct CompressedSpace {
    pub vs: VirtSpace,
}

impl CompressedSpace {
    pub fn new(vs: VirtSpace) -> Self {
        assert!(vs.reserved().size.value() <= Self::max_capacity().value());

        Self {
            vs
        }
    }
}

impl CompressedSpace {
    pub fn max_capacity() -> ByteSize {
        ByteSize(align_down!(32 * G - SLOT_SIZE.value(), VirtSpace::page_size().value()))
    }
}

impl CompressedSpace {
    #[inline(always)]
    pub fn base(&self) -> Address {
        self.vs.reserved().start as _
    } 

    #[inline(always)]
    pub fn encode<T>(&self, ptr: *const T) -> NarrowPtr {
        if ptr.is_null() { return NarrowPtr::null() }

        NarrowPtr((((ptr as Address) - self.base() + SLOT_SIZE.value()) >> LOG_BYTES_PER_WORD) as _)
    }

    #[inline(always)]
    pub fn decode<T>(&self, nptr: NarrowPtr) -> *const T {
        if nptr.is_null() { return null(); }

        (((nptr.value() as Address) << LOG_BYTES_PER_WORD) - SLOT_SIZE.value() + self.base()) as _
    }
}

#[derive(Debug)]
pub struct NarrowEncoder {
    base: Address
}

impl NarrowEncoder {
    pub fn new(base: Address) -> Self {
        Self {
            base: base
        }
    }
}

impl NarrowEncoder {
    #[inline(always)]
    pub fn encode<T>(&self, ptr: *const T) -> NarrowPtr {
        if ptr.is_null() { return NarrowPtr::null() }

        NarrowPtr((((ptr as Address) - self.base + SLOT_SIZE.value()) >> LOG_BYTES_PER_WORD) as _)
    }

    #[inline(always)]
    pub fn decode<T>(&self, nptr: NarrowPtr) -> *const T {
        if nptr.is_null() { return null(); }

        (((nptr.value() as Address) << LOG_BYTES_PER_WORD) - SLOT_SIZE.value() + self.base) as _
    }
}
