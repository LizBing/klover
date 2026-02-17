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


use std::fmt::Debug;

use region::{
    Allocation,
    Protection
};

use crate::{
    align_up,
    utils::global_defs::{
        ByteSize,
        HeapWord,
        WordSize
    }
};

use super::mem_region::MemRegion;

pub struct VirtSpace {
    guard: Allocation,

    reserved: MemRegion,

    commit_top: *const HeapWord,
    executable: bool,
}

impl Debug for VirtSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtSpace")
            .field("reserved", self.reserved())
            .field("executable", &self.executable)
            .field("Self::committed()", &self.committed());

        Ok(())
    }
}

impl VirtSpace {
    pub fn page_size() -> ByteSize {
        ByteSize(region::page::size())
    }
}

impl VirtSpace {
    pub fn reserved(&self) -> &MemRegion {
        &self.reserved
    }

    pub fn committed(&self) -> MemRegion {
        MemRegion::with_end(self.reserved().start, self.commit_top)
    }
}

impl VirtSpace {
    fn prot_helper(&self) -> region::Protection {
        if self.executable {
            Protection::READ_WRITE_EXECUTE
        } else {
            Protection::READ_WRITE
        }
    }
}

impl VirtSpace {
    pub fn new(size: ByteSize, executable: bool) -> Self {
        let aligned = ByteSize(align_up!(size.value(), Self::page_size().value()));

        let guard = region::alloc(aligned.value(), Protection::NONE).unwrap();
        let mr = MemRegion::with_size(guard.as_ptr(), aligned.into());

        Self {
            guard,

            reserved: mr.clone(),
            commit_top: mr.start,

            executable
        }
    }

    pub fn with_addr(addr: *const HeapWord, size: ByteSize, executable: bool) -> Self {
        let aligned = ByteSize(align_up!(size.value(), Self::page_size().value()));

        let guard = region::alloc_at(addr, aligned.value(), Protection::NONE).unwrap();

        Self {
            guard,

            reserved: MemRegion::with_size(addr, aligned.into()),
            commit_top: addr.into(),

            executable
        }
    }
}

impl VirtSpace {
    // Pretouch memory by invoking MemRegion::touch()
    pub fn expand_by(&mut self, size: ByteSize) -> bool {
        let aligned = WordSize::from(size);

        unsafe {
            let new_top = self.commit_top.add(aligned.value());
            if !self.reserved().contains(new_top) {
                return false;
            }

            region::protect(self.commit_top, ByteSize::from(aligned).value(), self.prot_helper()).unwrap();
            self.commit_top = new_top;
        }

        true
    }

    pub fn shrink_by(&mut self, size: ByteSize) -> bool {
        let aligned = WordSize::from(size);

        unsafe {
            let new_top = self.commit_top.sub(aligned.value());
            if !self.reserved().contains(new_top) {
                return false;
            }
        
            region::protect(new_top, ByteSize::from(aligned).value(), Protection::NONE).unwrap();
            self.commit_top = new_top;
        }

        true
    }
}
