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

use std::{mem::MaybeUninit, ptr::{null, null_mut}, sync::atomic::{AtomicPtr, Ordering}};

use crate::{heap_word_align_up, memory::mem_region::MemRegion, utils::global_defs::HeapWord};

#[derive(Debug)]
pub struct Bumper {
    _mr: MemRegion,
    _top: AtomicPtr<HeapWord>,
}

impl Bumper {
    pub fn new(mr: MemRegion) -> Self {
        Self {
            _mr: mr.clone(),
            _top: AtomicPtr::new(mr.start() as _)
        }
    }
}

impl Bumper {
    pub fn alloc<T: Sized>(&mut self) -> *mut MaybeUninit<T> {
        let top = self._top.get_mut();
        let byte_size = heap_word_align_up!(size_of::<T>());

        let new_top = unsafe { top.byte_add(byte_size) };
        if new_top >= self._mr.end() as _ {
            return null_mut();
        }

        let res = *top;
        *top = new_top as _;
        
        res as _
    }

    pub fn par_alloc<T: Sized>(&self) -> *mut MaybeUninit<T> {
        let byte_size = heap_word_align_up!(size_of::<T>());

        let mut top = self._top.load(Ordering::Relaxed);
        let res;
        loop {
            let new_top = unsafe { top.byte_add(byte_size) };
            if new_top >= self._mr.end() as _ {
                return null_mut();
            }

            match self._top.compare_exchange_weak(top, new_top, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => {
                    res = top;
                    break;
                }

                Err(x) => {
                    top = x;
                }
            }
        }

        res as _
    }
}
