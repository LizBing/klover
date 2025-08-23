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

use once_cell::unsync::OnceCell;

use crate::{engine::context::Context, memory::allocation::c_heap_alloc, metaspace::klass_allocator::KlassMemPool, utils::{easy_cell::EasyCell, global_defs::address}};

thread_local! {
    static TLS: ThrdLocalStorage = ThrdLocalStorage::new();
}

#[derive(Debug)]
struct ThrdLocalStorage {
    _kmp: KlassMemPool,
    _ctx: Context,
}

impl ThrdLocalStorage {
    fn new() -> Self {
        Self {
            _kmp: KlassMemPool::new(),
            _ctx: Context::new()
        }
    }
}

fn tls() -> &'static ThrdLocalStorage {
    TLS.with(|tls| {
        unsafe { &*(tls as *const _) }
    })
}

pub fn klass_mem_pool() -> &'static KlassMemPool {
    &tls()._kmp
}

pub fn context() -> &'static Context {
    &tls()._ctx
}
