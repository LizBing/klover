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

use crate::{engine::context::Context, memory::allocation::c_heap_alloc, metaspace::klass_allocator::KlassMemPool, utils::easy_cell::EasyCell};

thread_local! {
    static TLS: OnceCell<EasyCell<ThrdLocalStorage>> = OnceCell::new();
}

#[derive(Debug)]
struct ThrdLocalStorage {
    _kmp: KlassMemPool,
    _ctx: Context,
}

impl ThrdLocalStorage {
    fn init(&mut self) {
        *self = Self {
            _kmp: KlassMemPool::new(),
            _ctx: Context::new(),
        }
    }
}

pub fn initialize() {
    TLS.with(|tls| {
        let mem = c_heap_alloc(size_of::<ThrdLocalStorage>()).unwrap();
        let cell = EasyCell::new(mem.begin() as *mut ThrdLocalStorage);

        cell.get_mut().init();

        tls.set(cell).unwrap();
    })
}

pub fn klass_mem_pool() -> &'static mut KlassMemPool {
    TLS.with(|tls|{
        &mut tls.get().unwrap().get_mut()._kmp
    })
}

pub fn context() -> &'static Context {
    TLS.with(|tls| {
        &tls.get().unwrap().get()._ctx
    })
}
