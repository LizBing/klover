/*
 * Copyright 2026 Lei Zaakjyu
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

use std::{
    ops::{
        Deref,
        DerefMut
    },
    ptr::NonNull
};

#[derive(Debug)]
pub struct Handle<T: ?Sized + Sync> {
    raw: NonNull<T>
}

impl<T: Sync> Handle<T> {
    pub unsafe fn new(raw: NonNull<T>) -> Self {
        Self {
            raw: raw
        }
    }
}

impl<T: Sync> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self { raw: self.raw }
    }
}

impl<T: Sync> Handle<T> {
    fn raw(&self) -> NonNull<T> {
        self.raw
    }
}

impl<T: Sync> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.raw.as_ref()
        }
    }
}

impl<T: Sync> DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            self.raw.as_mut()
        }
    }
}
