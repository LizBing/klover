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

use std::cell::{Cell, OnceCell, RefCell, UnsafeCell};
use std::ptr::null_mut;
use crate::metaspace::klass_allocator::KlassMemPool;

thread_local! {
    static TLS: RefCell<Option<ThrdLocalStorage>> = RefCell::new(None);
}

struct ThrdLocalStorage {
    _kmp: KlassMemPool
}

impl ThrdLocalStorage {
    fn new() -> ThrdLocalStorage {
        Self {
            _kmp: KlassMemPool::new()
        }
    }
}

pub fn initialize() {
    TLS.set(Some(ThrdLocalStorage::new()));
}

pub fn klass_mem_pool() -> &'static mut KlassMemPool {
    TLS.with(|tls| &mut tls.borrow_mut().as_mut().unwrap()._kmp)
}
