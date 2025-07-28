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

use region::Protection;

use crate::{align_up, is_page_aligned, utils::global_defs::address};
use super::mem_region::MemRegion;

pub struct VirtSpace {
    _guard: Option<region::Allocation>,

    _commit_top: address,
    _executable: bool
}

fn commit_region(start: address, size: usize, exec: bool) -> bool {
    let mut prot = Protection::READ_WRITE;
    if exec { prot |= Protection::EXECUTE; }

    unsafe {
        match region::protect(start as *const c_void, size, prot) {
            Ok(_) => true,
            _ => false
        }
    }
}

fn uncommit_region(start: address, size: usize) -> bool {
    unsafe { 
        match region::protect(start as *const c_void, size, Protection::NONE) {
            Ok(_) => true,
            _ => false
        }
    }
}

impl VirtSpace {
    pub fn new(base: address,
               size: usize,
               init_commit: usize,
               executable: bool,
               pretouch: bool
        ) -> Result<Self, String> {
        assert!(is_page_aligned!(size));

        let mut vs = Self {
            _guard: None,
            _commit_top: 0,
            _executable: executable
        };

        let res = {
            if base == 0 { region::alloc(size, Protection::NONE) }
            else { region::alloc_at(base as *const c_void, size, Protection::NONE) }
        };

        match res {
            Ok(a) => {
                vs._guard = Some(a);
            }
            Err(e) => { return Err(e.to_string()); }
        }

        vs._commit_top = vs.mr().begin() + init_commit;
        if init_commit != 0 {
            commit_region(vs._commit_top, init_commit, executable);
            if pretouch {
                vs.committed_region().pretouch();
            }
        }

        Ok(vs)
    }
}

impl VirtSpace {
    pub fn page_size() -> usize {
        region::page::size()
    }
}

impl VirtSpace {
    pub fn mr(&self) -> MemRegion {
        let guard = self._guard.as_ref().unwrap();
        MemRegion::with_size(guard.as_ptr::<c_void>() as address, guard.len())
    }

    pub fn committed_region(&self) -> MemRegion {
        MemRegion::with_end(self.mr().begin(), self._commit_top)
    }
}

impl VirtSpace {
    pub fn expand_by(&mut self, _size: usize, exec: bool) -> usize {
        let size = align_up!(_size, Self::page_size());
        debug_assert!(self.mr().end() >= self._commit_top + size, "out of space");

        let prev_comt = self._commit_top;
        self._commit_top += size;

        if commit_region(prev_comt, size, exec) { size }
        else { 0 }
    }

    pub fn shrink_by(&mut self, size: usize) -> bool {
        assert!(is_page_aligned!(size));
        debug_assert!(self.mr().begin() <= self._commit_top - size, "out of space");

        self._commit_top -= size;

        uncommit_region(self._commit_top, size)
    }
}

