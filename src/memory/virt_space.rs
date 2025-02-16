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

use std::os::raw::c_void;


use crate::{is_aligned, runtime::os::{self, memadv_free, memmap, memprot, pretouch_region}, util::global_defs::address};

use super::mem_region::MemRegion;

pub struct VirtSpace {
    _region: MemRegion,

    _commit_top: address,
    _executable: bool
}

fn commit_region(start: address, size: usize, exec: bool) -> bool {
    memprot(MemRegion::with_size(start, size), true, exec)
}

fn uncommit_region(start: address, size: usize) -> bool {
    let mr = MemRegion::with_size(start, size);

    let res = memprot(mr, false, false);
    if res == false { return false; }

    memadv_free(mr)
}

impl VirtSpace {
    pub fn new(base: address,
               size: usize,
               init_commit: usize,
               executable: bool,
               pretouch: bool
        ) -> Result<Self, String> {
        assert!(is_aligned!(size, os::get_page_size()), "should be aligned");

        let mut vs = Self {
            _region: MemRegion::new(),
            _commit_top: 0,
            _executable: executable
        };

        match memmap(base, size) {
            Some(x) => vs._region = x,
            None => return Err(String::from("out of memory"))
        }

        vs._commit_top = vs._region.begin() + init_commit;

        commit_region(vs._commit_top, init_commit, executable);

        if pretouch {
            pretouch_region(&vs.committed_region());
        }

        Ok(vs)
    }
}

impl VirtSpace {
    pub fn mr(&self) -> MemRegion {
        self._region
    }

    pub fn committed_region(&self) -> MemRegion {
        MemRegion::with_end(self.mr().begin(), self._commit_top)
    }
}

impl VirtSpace {
    pub fn expand_by(&mut self, size: usize, exec: bool) -> bool {
        assert!(is_aligned!(size, os::get_page_size()), "should be aligned");
        debug_assert!(self.mr().end() >= self._commit_top + size, "out of space");

        let prev_comt = self._commit_top;
        self._commit_top += size;

        commit_region(prev_comt, size, exec)
    }

    pub fn shrink_by(&mut self, size: usize) -> bool {
        assert!(is_aligned!(size, os::get_page_size()), "should be aligned");
        debug_assert!(self.mr().begin() <= self._commit_top - size, "out of space");

        self._commit_top -= size;

        uncommit_region(self._commit_top, size)
    }
}

