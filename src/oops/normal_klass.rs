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

use std::sync::Arc;

use cafebabe::{parse_class, ClassFile};

use crate::{classfile::{class_loader::ClassLoader, class_loader_data::ClassLoaderData}, oops::{klass::{Klass, KlassBase}, oop_handle::OOPHandle, oop_hierarchy::OOP, weak_handle::WeakHandle}};

#[derive(Debug)]
pub struct NormalKlass {
    _next_klass: *const Klass,

    _name: String
}

impl KlassBase for NormalKlass {
    fn name(&self) -> &str {
        self._name.as_str()
    }

    fn _next_ptr(&self) -> *mut *const Klass {
        &self._next_klass as *const _ as _
    }
}

impl NormalKlass {
    pub fn new(stream: Vec<u8>) -> Result<Self, String> {
        unimplemented!()
    }
}
