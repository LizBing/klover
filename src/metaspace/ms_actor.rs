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

use std::{ptr::NonNull, sync::atomic::{AtomicPtr, Ordering}};

use tokio::sync::mpsc;

use crate::{memory::compressed_space::NarrowEncoder, metaspace::metaspace::{MSChunk, Metaspace}, utils::global_defs::{ByteSize, HeapWord}};

pub enum MSMsg {
    Shutdown,

    TryAndAllocateSmallChunk { slot: AtomicPtr<MSChunk>, size: ByteSize, reply_tx: mpsc::Sender<*const HeapWord> },

    AllocateSizedChunk { size: ByteSize, reply_tx: mpsc::Sender<NonNull<MSChunk>> },

    FreeChunk { chunk: NonNull<MSChunk> },
}

unsafe impl Send for MSMsg {}

pub struct MSActor {
    metaspace: Metaspace,
    rx: mpsc::UnboundedReceiver<MSMsg>
}

impl MSActor {
    pub fn new(rx: mpsc::UnboundedReceiver<MSMsg>) -> Self {
        let mut res = Self {
            metaspace: Metaspace::new(),
            rx: rx
        };

        res.metaspace.init();

        res
    }
}

impl MSActor {
    // forward
    pub fn create_narrow_encoder(&self) -> NarrowEncoder {
        self.metaspace.create_narrow_encoder()
    }
}

impl MSActor {
    pub async fn run(mut self) {
        loop {
            match self.rx.recv().await.unwrap() {
                MSMsg::Shutdown => break,

                MSMsg::TryAndAllocateSmallChunk { slot, size, reply_tx } => {
                    unsafe {
                        let chunk = slot.load(Ordering::Relaxed);
                        let attempt = (*chunk).bumper.par_alloc_with_size(size.into());
                        if !attempt.is_null() {
                            reply_tx.send(attempt).await.unwrap();
                        } else {
                            let new_chunk = self.metaspace.alloc_small_chunk().as_mut();

                            let res = new_chunk.bumper.alloc_with_size(size.into());
                            debug_assert!(!res.is_null());

                            slot.store(new_chunk, Ordering::Release);

                            reply_tx.send(res).await.unwrap()
                        }
                    }
                }

                MSMsg::AllocateSizedChunk { size, reply_tx } => {
                    reply_tx.send(self.metaspace.alloc_sized_chunk(size)).await.unwrap()
                }

                MSMsg::FreeChunk { chunk } => {
                    self.metaspace.free_chunk(chunk);
                }
            }
        }
    }
}
