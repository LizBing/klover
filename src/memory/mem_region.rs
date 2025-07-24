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

use std::ffi::c_void;

use libc::memset;

use crate::{is_page_aligned, utils::global_defs::{address, word_t}};

pub struct MemRegion {
    _begin: address,
    _end:   address
}

impl MemRegion {
    pub fn new() -> Self {
        Self {
            _begin: 0,
            _end: 0
        }
    }

    pub fn with_size(begin: address, size: usize) -> Self {
        Self::with_end(begin, begin + size)
    }

    pub fn with_end(begin: address, end: address) -> Self {
        let mut mr = Self::new();
        mr.init_with_end(begin, end);

        mr
    }

    pub fn init_with_size(&mut self, begin: address, size: usize) {
        self.init_with_end(begin, begin + size);
    }

    pub fn init_with_end(&mut self, begin: address, end: address) {
        debug_assert!(end >= begin, "bad memory region");

        self._begin = begin;
        self._end = end;
    }
}

impl Clone for MemRegion {
    fn clone(&self) -> Self {
        Self {
            _begin: self._begin,
            _end: self._end
        }
    }
}

impl Copy for MemRegion {}

impl MemRegion {
    pub fn assert_page_alignment(&self) {
        debug_assert!(is_page_aligned!(self._begin));
    }

    pub fn assert_available(&self) {
        debug_assert!(self._begin != 0 && self._end != 0);
    }
}

impl MemRegion {
    pub fn begin(&self) -> address {
        self._begin
    }

    pub fn end(&self) -> address {
        self._end
    }

    pub fn last_word(&self) -> address {
        self.end() - size_of::<word_t>()
    }

    pub fn size(&self) -> usize {
        self.end() - self.begin()
    }

    pub fn contains(&self, addr: address) -> bool {
        addr >= self.begin() && addr < self.end()
    }

    pub fn set_begin(&mut self, n: address) {
        self._begin = n;
    }

    pub fn set_end(&mut self, n: address) {
        self._end = n;
    }

    pub fn set_size(&mut self, s: usize) {
        self._end = self._begin + s;
    }
}

impl MemRegion {
    pub fn pretouch(&self) {
        self.assert_available();
        self.assert_page_alignment();

        unsafe {
            let ptr = self._begin as *mut u8;
            let page_size = region::page::size();

            for i in (0..self.size()).step_by(page_size) {
                ptr.add(i).write_volatile(0);
            }
        }
    }

    pub fn memset(&self, c: i32) {
        unsafe { memset(self._begin as *mut c_void, c, self.size()); }
    }
}
