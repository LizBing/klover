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

use std::ptr::null;

use crate::{gc::oop_storage::OOPStorage, utils::lock_free_stack::{LockFreeStack, NextPtr}};

static STORAGES: LockFreeStack<ThrdLocalStorage> = LockFreeStack::new();

thread_local! {
    static TLS: ThrdLocalStorage = ThrdLocalStorage::new();
}

pub struct ThrdLocalStorage {
    _next_ptr: *const Self,

    _oop_storage: OOPStorage
}

unsafe impl NextPtr for ThrdLocalStorage {
    fn next_ptr(&self) -> *mut *const Self {
        &self._next_ptr as *const _ as *mut _
    }
}

impl ThrdLocalStorage {
    fn new() -> Self {
        let res = Self {
            _next_ptr: null(),

            _oop_storage: OOPStorage::new(),
        };

        STORAGES.push(&res);

        res
    }
}

// Accessors
impl ThrdLocalStorage {
    fn tls() -> &'static ThrdLocalStorage {
        TLS.with(|n| -> &'static ThrdLocalStorage {
            unsafe {
                &*(n as *const _)
            }
        })
    }

    pub fn oop_storage() -> &'static OOPStorage {
        &Self::tls()._oop_storage
    }
}

