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

use std::{ptr::{null, null_mut}};

use crate::{memory::virt_space::VirtSpace, utils::global_defs::{HeapWord, LOG_BYTES_PER_WORD}};

#[derive(Clone, Debug)]
pub struct MemRegion {
    _start: *mut HeapWord,
    _word_size: usize
}

impl MemRegion {
    pub fn new() -> Self {
        Self {
            _start: null_mut(),
            _word_size: 0
        }
    }

    pub fn with_size<T: Into<*const HeapWord>>(start: T, word_size: usize) -> Self {
        Self {
            _start: start.into() as _,
            _word_size: word_size
        }
    }

    pub fn with_end<T: Into<*const HeapWord> + Copy>(start: T, end: T) -> Self {
        Self {
            _start: start.into() as _,
            _word_size: unsafe {
                end.into().offset_from_unsigned(start.into())
            }
        }
    }
}

impl MemRegion {
    pub fn start(&self) -> *const HeapWord {
        self._start
    }

    pub fn end(&self) -> *const HeapWord {
        unsafe { self._start.add(self._word_size) }
    }

    pub fn last_word(&self) -> *const HeapWord {
        unsafe { self.end().sub(1) }
    }

    pub fn size_in_words(&self) -> usize {
        self._word_size
    }

    pub fn size_in_bytes(&self) -> usize {
        self._word_size << LOG_BYTES_PER_WORD
    }

    pub fn contains<T: Into<*const HeapWord> + Copy>(&self, addr: T) -> bool {
        self._start <= addr.into() as _ && addr.into() < self.end() as _
    }

    pub fn set_start<T: Into<*const HeapWord>>(&mut self, n: T) {
        self._start = n.into() as _
    }

    pub fn set_end<T: Into<*const HeapWord>>(&mut self, n: T) {
        self._word_size = unsafe { n.into().offset_from_unsigned(self._start) };
    }

    pub fn set_size(&mut self, word_size: usize) {
        self._word_size = word_size
    }
}

impl MemRegion {
    pub unsafe fn touch(&self) {
        let step = VirtSpace::page_byte_size();

        let mut iter = self._start as *mut HeapWord;
        loop {
            if !self.contains(iter as *const _) { break; }

            iter.write_volatile(null());

            iter = iter.byte_add(step);
        }
    }

    pub unsafe fn memset(&self, b: u8) {
        std::ptr::write_bytes(self._start as *mut HeapWord, b, self.size_in_bytes());
    }
}
