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


use std::{cell::Cell, fmt::Debug};

use region::{Allocation, Protection};

use crate::{align_up, utils::global_defs::HeapWord};

use super::mem_region::MemRegion;

pub struct VirtSpace {
    _guard: Allocation,

    _reserved: MemRegion,
    _alignment: usize,

    _commit_top: *const HeapWord,
    _executable: bool
}

impl Debug for VirtSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtSpace")
            .field("_reserved", self.reserved())
            .field("_alignment", &self._alignment)
            .field("_executable", &self._executable)
            .field("(output of Self::committed())", &self.committed());

        Ok(())
    }
}

impl VirtSpace {
    pub fn page_byte_size() -> usize {
        region::page::size()
    }
}

impl VirtSpace {
    pub fn reserved(&self) -> &MemRegion {
        &self._reserved
    }

    pub fn committed(&self) -> MemRegion {
        MemRegion::with_end(self.reserved().start(), self._commit_top)
    }
}

impl VirtSpace {
    fn prot_helper(&self) -> region::Protection {
        if self._executable {
            Protection::READ_WRITE_EXECUTE
        } else {
            Protection::READ_WRITE
        }
    }
}

impl VirtSpace {
    pub fn new(mut word_size: usize, mut alignment: usize, executable: bool) -> Self {
        alignment = align_up!(alignment, Self::page_byte_size());
        word_size = align_up!(word_size, alignment);

        let guard = region::alloc(word_size * size_of::<HeapWord>(), Protection::NONE).unwrap();
    
        let mr = MemRegion::with_size(guard.as_ptr(), word_size);

        Self {
            _guard: guard,

            _reserved: mr.clone(),
            _alignment: alignment,

            _commit_top: mr.start(),

            _executable: executable
        }
    }

    pub fn with_addr<T: Into<*const HeapWord> + Copy>(addr: T, mut word_size: usize, mut alignment: usize, executable: bool) -> Self {
        alignment = align_up!(alignment, Self::page_byte_size());
        word_size = align_up!(word_size, alignment);

        let guard = region::alloc_at(addr.into(), word_size * size_of::<HeapWord>(), Protection::NONE).unwrap();

        Self {
            _guard: guard,

            _reserved: MemRegion::with_size(addr, word_size),
            _alignment: alignment,

            _commit_top: addr.into(),

            _executable: executable
        }
    }
}

impl VirtSpace {
    // Pretouch memory by invoking MemRegion::touch()
    pub fn expand_by(&mut self, mut word_size: usize) -> bool {
        word_size = align_up!(word_size, self._alignment);

        unsafe {
            let new_top = self._commit_top.add(word_size);
            if !self.reserved().contains(new_top) {
                return false;
            }

            region::protect(self._commit_top, word_size * size_of::<HeapWord>(), self.prot_helper()).unwrap();
            self._commit_top = new_top;
        }

        true
    }

    pub fn shrink_by(&mut self, mut word_size: usize) -> bool {
        word_size = align_up!(word_size, self._alignment);

        unsafe {
            let new_top = self._commit_top.sub(word_size);
            if !self.reserved().contains(new_top) {
                return false;
            }
        
            region::protect(new_top, word_size * size_of::<HeapWord>(), Protection::NONE).unwrap();
            self._commit_top = new_top;
        }

        true
    }
}
