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

use crate::class_data::java_classes::{JavaLangClass};
use crate::common::universe;
use crate::oops::klass::Klass;
use crate::oops::mark_word::MarkWord;
use crate::oops::obj_desc::ObjDesc;
use crate::oops::oop;
use crate::oops::oop::ObjPtr;
use crate::utils::global_defs::{addr_cast, address};

pub trait MemAllocator {
    fn size(&self) -> usize;

    fn klass(&self) -> &Klass<'static>;

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

pub struct ClassAllocator<'a> {
    _native: &'a Klass<'a>,
}

impl<'a> ClassAllocator<'a> {
    pub fn new(native: &'a Klass<'a>) -> Self {
        Self {
            _native: native,
        }
    }
}

impl<'a> MemAllocator for ClassAllocator<'a> {
    fn size(&self) -> usize {
        self._native.size_of_mirror()
    }

    fn klass(&self) -> &Klass<'static> {
        JavaLangClass::this()
    }

    fn initialize(&self, mem: address) {
        let slot_addr = mem + universe::heap().min_obj_size();
        *addr_cast::<&Klass>(slot_addr).unwrap() = self._native;
    } 
}

pub struct ObjAllocator<'a> {
    _klass: &'a Klass<'static>,
    _size: usize
}

impl<'a> ObjAllocator<'a> {
    pub fn new(klass: &'a Klass<'static>, size: usize) -> Self {
        Self {
            _klass: klass,
            _size: size
        }
    }
}

impl MemAllocator for ObjAllocator<'_> {
    fn size(&self) -> usize {
        self._size
    }

    fn klass(&self) -> &Klass<'static> {
        self._klass
    }

    fn initialize(&self, mem: address) { }
}

pub struct ArrayObjAllocator<'a> {
    _klass: &'a Klass<'static>,
    _size: usize,
    _len: i32
}

impl<'a> ArrayObjAllocator<'a> {
    pub fn new(klass: &'a Klass<'static>, size: usize, len: i32) -> Self {
        Self { _klass: klass, _size: size, _len: len }
    }
}

impl MemAllocator for ArrayObjAllocator<'_> {
    fn klass(&self) -> &Klass<'static> {
        self._klass
    }

    fn size(&self) -> usize {
        self._size
    }

    fn initialize(&self, mem: address) {
        ObjDesc::set_array_len(mem, self._len);
    }
}
