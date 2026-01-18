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

use std::{cell::{OnceCell, Ref, RefCell}, ptr::{null, null_mut}, sync::atomic::AtomicPtr};

use crate::{gc::oop_storage::OOPStorage, runtime::vm_thread::VMThread, utils::lock_free_stack::{LockFreeStack, NextPtr}};

static STORAGES: LockFreeStack<ThrdLocalStorage> = LockFreeStack::new();

thread_local! {
    static TLS: OnceCell<ThrdLocalStorage> = OnceCell::new();
}

#[derive(Debug)]
pub struct ThrdLocalStorage {
    _next_ptr: *mut Self,

    _thread: Box<dyn VMThread>
}

unsafe impl NextPtr<ThrdLocalStorage> for ThrdLocalStorage {
    fn _next_ptr(&self) -> *mut *mut Self {
        &self._next_ptr as *const _ as _
    }
}

impl ThrdLocalStorage {
    pub fn initialize<T: 'static + VMThread>(thrd: T) {
        TLS.with(|n| n.set(Self::new(thrd)).unwrap() )
    }

    fn new<T: 'static + VMThread>(thrd: T) -> Self {
        let res = Self {
            _next_ptr: null_mut(),

            _thread: Box::new(thrd)
        };

        STORAGES.push(&res);

        res
    }
}

// Accessors
impl ThrdLocalStorage {
    fn tls() -> &'static Self {
        TLS.with(|n| unsafe {
            &*(n.get().unwrap() as *const _)
        })
    }

    pub fn current_thread() -> &'static Box<dyn VMThread> {
        &Self::tls()._thread
    }
}

