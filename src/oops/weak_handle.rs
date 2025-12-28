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

use crate::{gc::oop_storage::OOPStorage, oops::oop_hierarchy::{NarrowOOP, OOP}};

#[derive(Debug)]
pub struct WeakHandle {}

impl WeakHandle {
    pub fn new() -> Self {
        unimplemented!()
    }

    pub fn with_storage(s: &OOPStorage) -> Self {
        unimplemented!()
    }

    pub fn with_oop(oop: OOP, s: &OOPStorage) -> Self {
        unimplemented!()
    }
}

impl WeakHandle {
    pub fn load(&self) -> Option<OOP> {
        unimplemented!()
    }
}
