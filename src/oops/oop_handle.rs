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

use crate::{gc::oop_storage::OOPStorage, oops::{access::{Access, DECORATOR_IN_NATIVE, DECORATOR_INTERNAL_NONCOMPRESSED, DECORATOR_MO_VOLATILE}, oop_hierarchy::OOP}};

#[derive(Debug)]
pub struct OOPHandle {
    _raw: NonNull<OOP>,
    _s: &'static OOPStorage
}

impl OOPHandle {
    pub fn new(s: &'static OOPStorage) -> Self {
        Self {
            _raw: s.allocate(),
            _s: s
        }
    }
}

impl Drop for OOPHandle {
    fn drop(&mut self) {
        self._s.free(self._raw);
    }
}

impl OOPHandle {
    pub fn load(&self) -> OOP {
        Access::<{DECORATOR_INTERNAL_NONCOMPRESSED | DECORATOR_IN_NATIVE | DECORATOR_MO_VOLATILE}>
            ::oop_load(self._raw.as_ptr())
    }

    pub fn store<const D: u32>(&self, n: OOP) {
        Access::<{DECORATOR_INTERNAL_NONCOMPRESSED | DECORATOR_IN_NATIVE | DECORATOR_MO_VOLATILE}>
            ::oop_store(self._raw.as_ptr(), n);
    }
}
