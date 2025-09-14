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


use std::{ffi::c_void, fs::File, num::NonZero, os::fd::AsFd, ptr::NonNull};

use nix::sys::mman::{mmap, mmap_anonymous, munmap, MapFlags, ProtFlags};

use crate::{align_up, is_page_aligned, utils::global_defs::address};
use super::mem_region::MemRegion;

pub struct VirtSpace {
    _mr: MemRegion,

    _commit_top: address,
    _executable: bool
}

fn commit_region(start: address, size: usize, exec: bool) -> bool {
    let mut prot = ProtFlags::PROT_READ | ProtFlags::PROT_WRITE;
    if exec { prot |= ProtFlags::PROT_EXEC; }

    unsafe {
        nix::sys::mman::mprotect(NonNull::new_unchecked(start as _), size, prot).is_ok()
    }
}

fn uncommit_region(start: address, size: usize) -> bool {
    unsafe { 
        nix::sys::mman::mprotect(NonNull::new_unchecked(start as _), size, ProtFlags::PROT_NONE).is_ok()
    }
}

impl VirtSpace {
    pub fn new(base: address,
               size: usize,
               init_commit: usize,
               fd: Option<&File>,
               offs: i64,
               exec: bool,
               pretouch: bool
        ) -> Self {
        assert!(size != 0);
        assert!(is_page_aligned!(size));
        assert!(is_page_aligned!(init_commit));

        let mut flags = MapFlags::MAP_PRIVATE;
        if base != 0 {
            flags |= MapFlags::MAP_FIXED;
        }

        let mm = match fd {
            Some(f) => unsafe { mmap(NonZero::new(base), NonZero::new(size).unwrap(), ProtFlags::PROT_NONE, flags, f.as_fd(), offs) },
            None => unsafe { mmap_anonymous(NonZero::new(base), NonZero::new(size).unwrap(), ProtFlags::PROT_NONE, flags) }
        }.unwrap();

        let mr = MemRegion::with_size(mm.addr().get(), size);
        let commit_top = mr.begin() + init_commit;
    
        if init_commit != 0 {
            assert!(commit_region(mr.begin(), init_commit, exec));
            if pretouch {
                mr.pretouch();
            }
        }

        Self {
            _mr: mr,
            _commit_top: commit_top,
            _executable: exec
        }
    }
}

impl Drop for VirtSpace {
    fn drop(&mut self) {
        unsafe {
            munmap(NonNull::new_unchecked(self.mr().begin() as _), self.mr().size()).unwrap()
        }
    }
}

impl VirtSpace {
    pub fn page_size() -> usize {
        unsafe { nix::libc::vm_page_size }
    }
}

impl VirtSpace {
    pub fn mr(&self) -> &MemRegion {
        &self._mr
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

