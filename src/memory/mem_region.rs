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

use std::{marker::PhantomData, ptr::{null, null_mut}};

use crate::{memory::virt_space::VirtSpace, utils::global_defs::{ByteSize, HeapWord, WordSize}};

#[derive(Clone, Debug)]
pub struct MemRegion {
    pub start: *const HeapWord,
    pub size: WordSize,

    __: PhantomData<()>     // Avoid being constructed by MemRegion {...}
}

impl MemRegion {
    pub fn new() -> Self {
        Self {
            start: null_mut(),
            size: WordSize(0),
            __: PhantomData
        }
    }

    pub fn with_size(start: *const HeapWord, size: WordSize) -> Self {
        Self {
            start: start,
            size: size,
            __: PhantomData
        }
    }

    pub fn with_end(start: *const HeapWord, end: *const HeapWord) -> Self {
        Self {
            start: start,
            size: WordSize(unsafe { end.offset_from_unsigned(start) }),
            __: PhantomData
        }
    }
}

impl MemRegion {
    pub fn end(&self) -> *const HeapWord {
        unsafe { self.start.add(self.size.value()) }
    }

    pub fn last_word(&self) -> *const HeapWord {
        unsafe { self.end().sub(1) }
    }

    pub fn contains<T>(&self, addr: *const T) -> bool {
        self.start <= addr as _ && addr < self.end() as _
    }

    pub fn set_end(&mut self, n: *const HeapWord) {
        self.size = WordSize(unsafe { n.offset_from_unsigned(self.start) });
    }
}

impl MemRegion {
    pub unsafe fn touch(&self) {
        let step = VirtSpace::page_size();

        let mut iter = self.start as *mut HeapWord;
        loop {
            if !self.contains(iter as *const _) { break; }

            iter.write_volatile(null());

            iter = iter.byte_add(step.value());
        }
    }

    pub unsafe fn memset(&self, b: u8) {
        std::ptr::write_bytes(self.start as *mut HeapWord, b, ByteSize::from(self.size).value());
    }
}
