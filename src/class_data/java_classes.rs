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
use std::cell::OnceCell;
use crate::common::universe;
use crate::gc::common::mem_allocator::{ClassAllocator, MemAllocator};
use crate::oops::klass::Klass;
use crate::oops::oop::ObjPtr;

static mut JAVA_LANG_OBJECT: OnceCell<&'static Klass> = OnceCell::new();
pub struct JavaLangObject;

impl JavaLangObject {
    pub fn this() -> &'static Klass<'static> {
        unsafe {
            JAVA_LANG_OBJECT.get().unwrap()
        }
    }

    pub fn size_of_instance() -> usize {
        universe::heap().min_obj_size()
    }
}

static mut JAVA_LANG_CLASS: OnceCell<&'static Klass> = OnceCell::new();
pub struct JavaLangClass;

impl JavaLangClass {
    pub fn this() -> &'static Klass<'static> {
        unsafe {
            JAVA_LANG_CLASS.get().unwrap()
        }
    }
}

impl JavaLangClass {
    pub fn new_instance(native: &'static Klass) -> ObjPtr {
        let allocator = ClassAllocator::new(native);
        allocator.allocate()
    }
}

pub fn initialize() {
    unimplemented!()
}
