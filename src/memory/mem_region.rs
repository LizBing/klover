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

use std::ptr::{null, null_mut};

use crate::utils::global_defs::{HeapAddress, LOG_BYTES_PER_WORD};

#[derive(Clone, Debug)]
pub struct MemRegion {
    _start: HeapAddress,
    _word_size: usize
}

impl MemRegion {
    pub fn new() -> Self {
        Self {
            _start: HeapAddress::new(null()),
            _word_size: 0
        }
    }

    pub fn with_size(start: HeapAddress, word_size: usize) -> Self {
        Self {
            _start: start,
            _word_size: word_size
        }
    }

    pub fn with_end(start: HeapAddress, end: HeapAddress) -> Self {
        Self {
            _start: start,
            _word_size: HeapAddress::delta_in_words(end, start)
        }
    }
}

impl MemRegion {
    pub fn start(&self) -> HeapAddress {
        self._start
    }

    pub fn end(&self) -> HeapAddress {
        self.start().offset_in_words(self._word_size as _)
    }

    pub fn last_word(&self) -> HeapAddress {
        self.end().offset_in_words(-1)
    }

    pub fn size_in_words(&self) -> usize {
        self._word_size
    }

    pub fn size_in_bytes(&self) -> usize {
        self._word_size << LOG_BYTES_PER_WORD
    }

    pub fn contains(&self, addr: HeapAddress) -> bool {
        HeapAddress::diff_in_bytes(self._start, addr) <= 0 &&
        HeapAddress::diff_in_bytes(self.end(), addr) > 0
    }

    pub fn set_begin(&mut self, n: HeapAddress) {
        self._start = n
    }

    pub fn set_end(&mut self, n: HeapAddress) {
        self._word_size = HeapAddress::delta_in_words(n, self._start)
    }

    pub fn set_size(&mut self, word_size: usize) {
        self._word_size = word_size
    }
}


