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
use crate::class_data::java_classes::{initialize, JavaLangClass};
use crate::common::universe;
use crate::metaspace::klass_cell::KlassCell;
use crate::oops::mark_word::MarkWord;
use crate::oops::obj_desc::ObjDesc;
use crate::oops::oop;
use crate::oops::oop::ObjPtr;
use crate::utils::global_defs::{addr_cast, address};

pub trait MemAllocator {
    fn size(&self) -> usize;

    fn klass(&self) -> KlassCell;

    fn initialize(&self, mem: address);

    fn raw_alloc(&self) -> address {
        universe::heap().mem_alloc(self.size())
    }

    fn finish(&self, mem: address) -> ObjPtr {
        ObjDesc::set_mark(mem, MarkWord::prototype(self.klass()));
        oop::as_oop(mem)
    }

    fn allocate(&self) -> ObjPtr {
        let mem = self.raw_alloc();
        self.initialize(mem);
        self.finish(mem)
    }
}

pub struct ClassAllocator {
    _native: KlassCell
}

impl ClassAllocator {
    pub fn new(native: KlassCell) -> Self {
        Self {
            _native: native
        }
    }
}

impl MemAllocator for ClassAllocator {
    fn size(&self) -> usize {
        universe::heap().min_obj_size() + size_of::<KlassCell>()
    }

    fn klass(&self) -> KlassCell {
        JavaLangClass::this()
    }

    fn initialize(&self, mem: address) {
        let slot_addr = mem + universe::heap().min_obj_size();
        *addr_cast::<KlassCell>(slot_addr).unwrap() = self._native.clone();
    } 
}

