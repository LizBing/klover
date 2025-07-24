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

use std::{os::raw::c_void, ptr::null_mut};

use crate::{memory::mem_region::MemRegion, utils::global_defs::address};

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(addr: *mut c_void);
}

pub fn c_heap_alloc(size: usize) -> Option<MemRegion> {
    unsafe {
        let mem = malloc(size);
        if mem == null_mut() { return None; }

        Some(MemRegion::with_size(mem as address, size))
    }
}

pub fn c_heap_free(mr: MemRegion) {
    unsafe { free(mr.begin() as *mut c_void); }
}
