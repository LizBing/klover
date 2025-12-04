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

use crate::oops::{klass::Klass, oop_hierarchy::OOP};

pub trait MemAllocator {
    fn word_size(&self) -> usize;

    fn initialize(&self);

    fn allocate(&self) -> OOP {
        unimplemented!()
    }
}

pub struct ObjAllocator {
    _word_size: usize
}

impl ObjAllocator {
    pub fn new(klass: &Klass, word_size: usize) -> Self {
        unimplemented!()
    }
}

impl MemAllocator for ObjAllocator {
    fn word_size(&self) -> usize {
        self._word_size
    }

    fn initialize(&self) {
        unimplemented!()
    }
}

pub struct ObjArrayAllocator {
    _word_size: usize,
    _length: i32,
    _do_zero: bool
}

impl ObjArrayAllocator {
    pub fn new(klass: &Klass, word_size: usize, length: i32, do_zero: bool) -> Self {
        unimplemented!()
    }
}

impl MemAllocator for ObjArrayAllocator {
    fn word_size(&self) -> usize {
        self._word_size
    }

    fn initialize(&self) {
        unimplemented!()
    }
}

