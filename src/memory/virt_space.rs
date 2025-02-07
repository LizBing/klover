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

use nix::libc::{mmap, mprotect, MAP_ANON, MAP_FAILED, MAP_FIXED, MAP_PRIVATE, PROT_EXEC, PROT_NONE, PROT_READ, PROT_WRITE};

use crate::{is_aligned, runtime::os::{self, pretouch_region}, util::global_defs::address};

use super::mem_region::MemRegion;

pub struct VirtSpace {
    _region: MemRegion,

    _commit_top: address,
    _executable: bool
}

fn commit_region(start: address, size: usize, exec: bool) -> bool {
    let mut prot = PROT_READ | PROT_WRITE;
    if exec {
        prot |= PROT_EXEC;
    }

    let res = unsafe { mprotect(start as *mut _, size, prot) };
    return res == 0;
}

impl VirtSpace {
    pub fn new(base: address,
               size: usize,
               align: usize,
               page_size: usize,
               init_commit: usize,
               executable: bool,
               pretouch: bool
        )
    -> Result<Self, String> {
        assert!(is_aligned!(base, page_size) && is_aligned!(size, page_size),
                "should be aligned.");

        let mut flags = MAP_ANON | MAP_PRIVATE;
        if base != 0 {
            flags |= MAP_FIXED;
        }

        let mut vs = VirtSpace {
            _region: MemRegion::new(),
            _commit_top: 0,
            _executable: executable
        };

        let begin = unsafe { mmap(base as *mut c_void, size, PROT_NONE, flags, -1, 0) as address };
        if begin == MAP_FAILED as address {
            return Err(String::from("failed to mmap"));
        }

        vs._region.init_with_size(begin, size);
        vs._commit_top = begin + init_commit;

        commit_region(begin, init_commit, executable);

        if pretouch {
            pretouch_region(&vs.committed_region());
        }

        Ok(vs)
    }
}

impl VirtSpace {
    pub fn mr(&self) -> &MemRegion {
        &self._region
    }

    pub fn committed_region(&self) -> MemRegion {
        MemRegion::with_end(self.mr().begin(), self._commit_top)
    }
}

