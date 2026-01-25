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

use std::ptr::{NonNull, null};

use crate::{oops::{klass::Klass, oop_hierarchy::OOP, weak_handle::WeakHandle}, utils::lock_free_stack::{LockFreeStack, NextPtr}};

#[derive(Debug)]
pub struct ClassLoaderData {
    _next_cld: *const Self,

    _mirror: WeakHandle,
    _klasses: LockFreeStack<Klass>
}

unsafe impl NextPtr<ClassLoaderData> for ClassLoaderData {
    fn _next_ptr(&self) -> *mut *const ClassLoaderData {
        &self._next_cld as *const _ as _
    }
}

impl ClassLoaderData {
    pub fn new(loader: OOP) -> Self {
        unimplemented!()
    }
}

impl ClassLoaderData {
    // returns false if duplicated
    pub fn register(&self, name: String, klass: NonNull<Klass>) -> bool {
        unimplemented!()
    }
}
