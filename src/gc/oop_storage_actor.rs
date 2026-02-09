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

use std::{array::from_fn, ptr::NonNull};
use tokio::sync::mpsc;

use crate::{gc::oop_storage::OOPStorage, oops::oop_hierarchy::OOP};

const ALL_START: usize = 0;
const STRONG_START: usize = ALL_START;
const STRONG_COUNT: usize = 1;
const WEAK_START: usize = STRONG_START + STRONG_COUNT;
const WEAK_COUNT: usize = 0;
const ALL_COUNT: usize = STRONG_COUNT + WEAK_COUNT;

pub const CLD_STORAGE_INDEX: usize = STRONG_START + 0;

pub enum OOPStorageMsg {
    Allocate { index: usize, reply_tx: mpsc::Sender<NonNull<OOP>> },

    Free { index: usize, addr: NonNull<OOP> },

    Shutdown
}

unsafe impl Send for OOPStorageMsg {}

pub struct OOPStorageActor {
    rx: mpsc::UnboundedReceiver<OOPStorageMsg>,
    array: [OOPStorage; ALL_COUNT]
}

impl OOPStorageActor {
    pub fn new(rx: mpsc::UnboundedReceiver<OOPStorageMsg>) -> Self {
        Self {
            rx: rx,
            array: from_fn(|_| OOPStorage::new())
        }
    }
}

impl OOPStorageActor {
    pub async fn run(mut self) {
        loop {
            match self.rx.recv().await.unwrap() {
                OOPStorageMsg::Allocate { index, reply_tx } => {
                    reply_tx.blocking_send(self.array[index].allocate()).unwrap()
                }

                OOPStorageMsg::Free { index, addr } => {
                    self.array[index].free(addr);
                }

                OOPStorageMsg::Shutdown => break
            }
        }
    }
}
