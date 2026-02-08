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

use std::ptr::{NonNull, null_mut};
use tokio::sync::mpsc;

use crate::{gc::oop_storage_actor::OOPStorageMsg, oops::{access::{Access, DECORATOR_IN_NATIVE, DECORATOR_INTERNAL_NONCOMPRESSED, DECORATOR_MO_VOLATILE}, oop_hierarchy::{NarrowOOP, OOP}}, runtime::universe::Universe};

#[derive(Debug)]
pub struct WeakHandle {
    raw: *mut OOP,
    storage_index: usize
}

impl WeakHandle {
    pub fn new() -> Self {
        Self {
            raw: null_mut(),
            storage_index: 0
        }
    }

    pub async fn with_storage(storage_index: usize) -> Self {
        let mut res = Self::new();
        res.init(storage_index).await;

        res
    }

    pub async fn init(&mut self, storage_index: usize) {
        let (tx, mut rx) = mpsc::channel(1);
        let msg = OOPStorageMsg::Allocate { index: storage_index, reply_tx: tx };

        Universe::actor_mailboxes().send_oop_storage(msg);

        *self = Self {
            raw: unsafe { rx.recv().await.unwrap().as_mut() },
            storage_index: storage_index
        };
    }
}

impl Drop for WeakHandle {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            let msg = OOPStorageMsg::Free { index: self.storage_index, addr: unsafe { NonNull::new_unchecked(self.raw) } };
            Universe::actor_mailboxes().send_oop_storage(msg);
        }
        
    }
}

impl WeakHandle {
    pub fn load(&self) -> OOP {
        Access::<{DECORATOR_INTERNAL_NONCOMPRESSED | DECORATOR_IN_NATIVE | DECORATOR_MO_VOLATILE}>
            ::oop_load(self.raw)
    }

    pub fn store(&self, n: OOP) {
        Access::<{DECORATOR_INTERNAL_NONCOMPRESSED | DECORATOR_IN_NATIVE | DECORATOR_MO_VOLATILE}>
            ::oop_store(self.raw, n);
    }
}
