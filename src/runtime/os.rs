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

// os extension

use std::sync::atomic::{AtomicI32, Ordering};

use nix::libc::{madvise, mmap, mprotect, sysconf, MADV_FREE, MAP_ANON, MAP_FAILED, MAP_FIXED, MAP_PRIVATE, PROT_EXEC, PROT_NONE, PROT_READ, PROT_WRITE, _SC_PAGE_SIZE};

use crate::{memory::mem_region::MemRegion, util::global_defs::address};

pub fn memmap(addr: address, size: usize) -> Option<MemRegion> {
    let mut flags = MAP_ANON | MAP_PRIVATE;
    if addr != 0 { flags |= MAP_FIXED; }

    let res = unsafe {
        mmap(addr as *mut _, size, PROT_NONE, flags, -1, 0)
    };

    if res == MAP_FAILED { return None; }

    Some(MemRegion::with_size(res as _, size))
}

pub fn memprot(mr: MemRegion, accessible: bool, executable: bool) -> bool {
    let mut prots = PROT_NONE;
    if accessible { prots = PROT_READ | PROT_WRITE; }
    if executable { prots |= PROT_EXEC; }

    let res = unsafe {
        mprotect(mr.begin() as _, mr.size(), prots)
    };

    return res == 0;
}

pub fn memadv_free(mr: MemRegion) -> bool {
    let res = unsafe { madvise(mr.begin() as _, mr.size(), MADV_FREE) };
    return res == 0;
}

pub fn get_page_size() -> usize {
    unsafe { sysconf(_SC_PAGE_SIZE) as _ }
}

pub fn pretouch_region(mr: &MemRegion) {
    mr.assert_page_alignment();

    let page_size = get_page_size();
    let mut iter = mr.begin();
    while iter < mr.end() {
        let atom = unsafe { &mut *(iter as *mut AtomicI32) };
        atom.fetch_add(0, Ordering::Relaxed);

        iter += page_size;
    }
}
