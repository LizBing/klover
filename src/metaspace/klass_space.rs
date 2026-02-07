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

use std::{mem::MaybeUninit, ops::Add, ptr::NonNull, sync::OnceLock};

use crate::{memory::{bumper::Bumper, compressed_space::CompressedSpace, virt_space::VirtSpace}, oops::klass::Klass, utils::global_defs::{Address, HeapWord, M}};

pub type NarrowKlassPtr = u32;

const FIXED_SIZE: usize = 64 * M / size_of::<HeapWord>();

static KLASS_SPACE: OnceLock<KlassSpace> = OnceLock::new();

#[derive(Debug)]
pub struct KlassSpace {
    _bumper: Bumper,
    _cs: CompressedSpace,
}

unsafe impl Send for KlassSpace {}
unsafe impl Sync for KlassSpace {}

impl KlassSpace {
    fn new() -> Self {
        let mut vs = VirtSpace::new(FIXED_SIZE, VirtSpace::page_byte_size(), false);
        assert!(vs.expand_by(FIXED_SIZE));

        Self {
            _bumper: Bumper::new(vs.committed()),
            _cs: CompressedSpace::new(vs)
        }
    }

    pub fn initialize() {
        KLASS_SPACE.set(Self::new()).unwrap();
    }
}

impl KlassSpace {
    pub fn space() -> &'static Self {
        KLASS_SPACE.get().expect("Should be initialized in advance.")
    }

    pub fn cs(&self) -> &CompressedSpace {
        &self._cs
    }
}

impl KlassSpace {
    pub fn par_alloc(&self) -> *mut MaybeUninit<Klass> {
        let mem = self._bumper.par_alloc();
        assert!(!mem.is_null(), "out of memory(metaspace).");

        mem
    }
}
