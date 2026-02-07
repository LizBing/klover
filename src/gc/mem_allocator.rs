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

use std::ptr::NonNull;

use crate::{oops::{klass::Klass, obj_desc::{ArrayObjDesc, ObjDesc}, oop_hierarchy::OOP}, utils::global_defs::word_size_of};

pub struct MemAllocator {
    word_size: usize,
    klass: NonNull<Klass>,

    is_array: bool,
    length: usize,

    do_zero: bool,
}

impl MemAllocator {
    pub fn new(klass: NonNull<Klass>, do_zero: bool) -> Self {
        let word_size = unsafe { word_size_of::<ObjDesc>() + klass.as_ref().unit_word_size() };

        Self {
            word_size: word_size,
            klass: klass,
            is_array: false,
            length: 0,
            do_zero: do_zero
        }
    }

    pub fn new_array(klass: NonNull<Klass>, len: usize) -> Self {
        let word_size = unsafe { word_size_of::<ArrayObjDesc>() + klass.as_ref().unit_word_size() * len };

        Self {
            word_size: word_size,
            klass: klass,
            is_array: true,
            length: len,
            do_zero: true
        }
    }
}

impl MemAllocator {
    pub fn allocate(&self) -> OOP {
        unimplemented!()
    }
}
