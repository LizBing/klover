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

pub struct EasyCell<T: 'static> {
    _raw: *mut T
}

unsafe impl<T> Send for EasyCell<T> {}
unsafe impl<T> Sync for EasyCell<T> {}

impl<T> EasyCell<T> {
    pub fn new() -> Self {
        Self::with_raw(null_mut())
    }

    pub fn with_raw(raw: *mut T) -> Self {
        Self { _raw: raw }
    }
}

impl<T> Clone for EasyCell<T> {
    fn clone(&self) -> Self {
        Self::with_raw(self._raw)
    }
}

impl<T> EasyCell<T> {
    pub fn raw(&self) -> *mut T { self._raw }
}

impl<T> EasyCell<T> {
    pub fn get(&self) -> &'static T {
        unsafe {
            &*self._raw
        }
    }

    pub fn get_mut(&self) -> &'static mut T {
        unsafe {
            &mut *self._raw
        }
    }
}

