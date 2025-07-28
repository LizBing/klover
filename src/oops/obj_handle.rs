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

use crate::oops::oop::ObjPtr;

#[derive(Debug)]
pub struct ObjHandle {
    _oop: ObjPtr
}

unsafe impl Send for ObjHandle {}
unsafe impl Sync for ObjHandle {}

impl ObjHandle {
    pub fn new() -> Self {
        Self {
            _oop: null_mut()
        }
    }
    
    pub fn with_oop(oop: ObjPtr) -> Self {
        Self { _oop: oop }
    }
}

impl ObjHandle {
    pub fn oop(&self) -> ObjPtr {
        self._oop
    }

    pub fn set_oop(&mut self, oop: ObjPtr) {
        self._oop = oop;
    }
}
