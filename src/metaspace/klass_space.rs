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

use std::{ptr::NonNull};

use crate::{memory::{bumper::Bumper, virt_space::VirtSpace}, oops::klass::Klass, utils::global_defs::{HeapWord, M}};

pub type NarrowKlassPtr = u32;

const FIXED_SIZE: usize = 64 * M / size_of::<HeapWord>();


pub struct KlassSpace {
    _bumper: Bumper,
    _vm: VirtSpace,
}

impl KlassSpace {
    pub fn new() -> Self {
        let mut vm = VirtSpace::new(FIXED_SIZE, VirtSpace::page_size(), false);
        assert!(vm.expand_by(FIXED_SIZE));

        Self {
            _bumper: Bumper::new(vm.committed()),
            _vm: vm
        }
    }
}

impl KlassSpace {
    pub fn par_alloc(&self, data: Klass) -> NonNull<Klass> {
        let mem = self._bumper.par_alloc();
        assert!(!mem.is_null(), "out of memory(metaspace).");

        unsafe {
            NonNull::new_unchecked((*mem).write(data))
        }
    }
}
