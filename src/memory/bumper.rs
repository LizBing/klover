/*
 * Copyright 2026 Lei Zaakjyu
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

use std::{
    mem::MaybeUninit,
    ptr::null_mut,
    sync::atomic::{
        AtomicPtr,
        Ordering
    }
};

use crate::{
    memory::mem_region::MemRegion,
    utils::global_defs::{
        ByteSize,
        HeapWord,
        WordSize
    }
};

#[derive(Debug)]
pub struct Bumper {
    mr: MemRegion,
    top: AtomicPtr<HeapWord>,
}

impl Bumper {
    pub fn new(mr: MemRegion) -> Self {
        Self {
            mr: mr.clone(),
            top: AtomicPtr::new(mr.start as _)
        }
    }
}

impl Bumper {
    pub fn mr(&self) -> &MemRegion {
        &self.mr
    }

    pub fn allocated(&self) -> WordSize {
        MemRegion::with_end(self.mr.start, self.top.load(Ordering::Relaxed)).size
    }

    pub fn remaining(&self) -> WordSize {
        MemRegion::with_end(self.top.load(Ordering::Relaxed), self.mr.end()).size
    }
}

impl Bumper {
    pub fn clear(&mut self) {
        *self.top.get_mut() = self.mr.start as _
    }

    pub fn alloc_with_size(&mut self, size: WordSize) -> *mut HeapWord {
        let top = self.top.get_mut();

        let new_top = unsafe { top.add(size.value()) };
        if new_top >= self.mr.end() as _ {
            return null_mut();
        }

        let res = *top;
        *top = new_top as _;
        
        res
    }
    
    pub fn alloc<T: Sized>(&mut self) -> *mut MaybeUninit<T> {
        self.alloc_with_size(ByteSize(size_of::<T>()).into()) as _
    }

    pub fn par_alloc_with_size(&self, size: WordSize) -> *mut HeapWord {
        let mut top = self.top.load(Ordering::Relaxed);
        let res;
        loop {
            let new_top = unsafe { top.add(size.value()) };
            if new_top >= self.mr.end() as _ {
                return null_mut();
            }

            match self.top.compare_exchange_weak(top, new_top, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => {
                    res = top;
                    break;
                }

                Err(x) => {
                    top = x;
                }
            }
        }

        res
    }

    pub fn par_alloc<T: Sized>(&self) -> *mut MaybeUninit<T> {
        self.par_alloc_with_size(ByteSize(size_of::<T>()).into()) as _
    }
}
