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

use crate::{is_page_aligned, util::global_defs::address};

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
}

impl MemRegion {
    pub fn begin(&self) -> address {
        self._begin
    }

    pub fn end(&self) -> address {
        self._end
    }

    pub fn size(&self) -> usize {
        self.end() - self.begin()
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

