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
use std::ptr::null_mut;
use crate::class_data::java_classes::JavaLangClass;
use crate::common::universe;
use crate::oops::klass::{Klass, KlassHandle};
use crate::oops::mark_word::MarkWord;
use crate::oops::oop;
use crate::oops::oop::ObjPtr;
use crate::utils::global_defs::{addr_cast, address};

pub struct MemAllocator {
    _klass: Option<KlassHandle>,
    _size: usize,
}

impl MemAllocator {
    pub fn new(klass: Option<KlassHandle>, size: usize) -> Self {
        Self { _klass: klass, _size: size }
    }
}

impl MemAllocator {
    // Construct Markword.
    fn initialize(&self, mem: address) {
        let oop = oop::as_oop(mem);
        unsafe {
            (*oop).init(self._klass.clone());
        }
    }

    pub fn raw_alloc(&self) -> address {
        universe::heap().mem_alloc(self._size)
    }

    pub fn allocate(&self) -> ObjPtr {
        let mem = self.raw_alloc();
        if mem == 0 { return null_mut(); }
        
        self.initialize(mem);
        oop::as_oop(mem)
    }
}


