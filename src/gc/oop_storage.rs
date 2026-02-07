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

use crate::oops::oop_hierarchy::{NarrowOOP, OOP};

#[derive(Debug)]
pub struct OOPStorage;

// temporary implementation
impl OOPStorage {
    pub fn new() -> Self {
        Self
    }

    pub fn allocate(&self) -> NonNull<OOP> {
        unsafe { NonNull::new_unchecked(Box::leak(Box::new(OOP::null()))) }
    }

    pub fn free(&self, mut n: NonNull<OOP>) {
        unsafe {
            Box::from_raw(n.as_mut());
        }
    }
}
