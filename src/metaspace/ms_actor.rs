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
    ops::DerefMut,
    ptr::NonNull
};

use tokio::sync::mpsc;

use crate::{
    classfile::class_loader_data::CLDHandle,
    memory::compressed_space::NarrowEncoder,
    metaspace::metaspace::{
        MSChunk,
        Metaspace
    },
    utils::global_defs::{
        ByteSize,
        HeapWord
    }
};

pub enum MSMsg {
    Shutdown,

    TryAndAllocateSmallChunk { cld: CLDHandle, size: ByteSize, reply_tx: mpsc::Sender<NonNull<HeapWord>> },

    AllocateSizedChunk { cld: CLDHandle, size: ByteSize, reply_tx: mpsc::Sender<NonNull<MSChunk>> },

    FreeChunks { cld: CLDHandle },
}

unsafe impl Send for MSMsg {}

pub struct MSActor {
    metaspace: Metaspace,
    rx: mpsc::UnboundedReceiver<MSMsg>
}

unsafe impl Send for MSActor {}

impl MSActor {
    pub fn new(rx: mpsc::UnboundedReceiver<MSMsg>, ms_size: ByteSize) -> Self {
        let mut res = Self {
            metaspace: Metaspace::new(ms_size),
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

                MSMsg::TryAndAllocateSmallChunk { mut cld, size, reply_tx } => {
                    let res = self.metaspace.try_and_alloc_small_chunk(cld.deref_mut(), size);
                    reply_tx.blocking_send(res).unwrap()
                }

                MSMsg::AllocateSizedChunk { mut cld, size, reply_tx } => {
                    let mut chunk = self.metaspace.alloc_sized_chunk(size);
                    unsafe {
                        cld.claim_new_sized_chunk(chunk.as_mut());
                    }

                    reply_tx.blocking_send(chunk).unwrap()
                }

                MSMsg::FreeChunks { mut cld } => {
                    cld.drop_chunks(|x| self.metaspace.free_chunk(x));
                }
            }
        }
    }
}
