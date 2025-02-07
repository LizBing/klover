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

use nix::libc::{sysconf, _SC_PAGE_SIZE};

use crate::memory::mem_region::MemRegion;

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
