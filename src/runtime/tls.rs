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

use std::{cell::RefCell, ptr::null_mut};
use once_cell::unsync::OnceCell;

use crate::{metaspace::klass_allocator::KlassMemPool, utils::easy_cell::EasyCell};

thread_local! {
    static TLS: OnceCell<EasyCell<ThrdLocalStorage>> = OnceCell::new();
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
    TLS.with(|tls| {
        tls.set(EasyCell::with_raw(Box::into_raw(Box::new(ThrdLocalStorage::new()))))
    });
}

pub fn klass_mem_pool() -> &'static mut KlassMemPool {
    TLS.with(|tls|{
        &mut tls.get().unwrap().get_mut()._kmp
    })
}
