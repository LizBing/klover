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

use std::{ptr::{NonNull, null_mut}, sync::{atomic::{AtomicPtr, Ordering}}};

use tokio::sync::mpsc;

use crate::{gc::oop_storage_actor::CLD_STORAGE_INDEX, init_ll, metaspace::{metaspace::{MSChunk, SMALL_MSCHUNK_SIZE}, ms_actor::MSMsg}, oops::{klass::Klass, oop_handle::OOPHandle, oop_hierarchy::OOP, symbol::Symbol}, runtime::universe::Universe, utils::{global_defs::{ByteSize, HeapWord}, linked_list::{LinkedList, LinkedListNode}}};

#[derive(Debug)]
pub struct ClassLoaderData {
    pub cld_graph_node: LinkedListNode<Self>,

    pub mirror: OOPHandle,
    klasses: LinkedList<Klass>,
    klass_count: usize,

    owned_chunks: LinkedList<MSChunk>,
    current_chunk: AtomicPtr<MSChunk>,
}

impl ClassLoaderData {
    pub fn init(&mut self, loader: OOP) {
        *self = Self {
            cld_graph_node: LinkedListNode::new(),

            mirror: OOPHandle::with_storage(CLD_STORAGE_INDEX),
            klasses: LinkedList::new(),
            klass_count: 0,

            owned_chunks: LinkedList::new(),
            current_chunk: AtomicPtr::new(null_mut()),
        };
        init_ll!(&mut self.klasses, Klass, cld_node);
        init_ll!(&mut self.owned_chunks, MSChunk, owning_node);

        self.mirror.store(loader);
    }

    pub fn init_bootstrap(&mut self) {
        *self = Self {
            cld_graph_node: LinkedListNode::new(),

            mirror: OOPHandle::new(),
            klasses: LinkedList::new(),
            klass_count: 0,

            owned_chunks: LinkedList::new(),
            current_chunk: AtomicPtr::new(null_mut()),
        };

        init_ll!(&mut self.klasses, Klass, cld_node);
        init_ll!(&mut self.owned_chunks, MSChunk, owning_node);
    }
}

impl ClassLoaderData {
    // returns false if duplicated
    pub fn register_klass(&mut self, mut klass: NonNull<Klass>) -> bool {
        if self.klasses.iterate(|iter| -> Option<()> {
            if iter.value().name() as *const Symbol == unsafe { klass.as_ref().name() } {
                Some(())
            } else { None }
        }).is_some() {
            return false;
        }

        unsafe { self.klasses.push_back(klass.as_mut()); }
        self.klass_count += 1;

        true
    }
}

impl ClassLoaderData {
    pub fn mem_alloc_with_size(&self, size: ByteSize) -> NonNull<HeapWord> {
        let res = self.try_mem_alloc(size, Ordering::Acquire);
        if !res.is_null() {
            return unsafe { NonNull::new_unchecked(res) };
        }

        let handle = unsafe { NonNull::new_unchecked(self as *const _ as _) };
        if size.value() < SMALL_MSCHUNK_SIZE.value() / 2 {
            let (tx, mut rx) = mpsc::channel(1);
            
            let msg = MSMsg::TryAndAllocateSmallChunk { cld: handle, size, reply_tx: tx };
            Universe::actor_mailboxes().send_metaspace(msg);
            
            rx.blocking_recv().unwrap()
        } else {
            let (tx, mut rx) = mpsc::channel(1);

            let msg = MSMsg::AllocateSizedChunk { cld: handle, size, reply_tx: tx };
            Universe::actor_mailboxes().send_metaspace(msg);
            let new_chunk = unsafe { rx.blocking_recv().unwrap().as_mut() };
            
            NonNull::new(new_chunk.bumper.alloc_with_size(size.into())).unwrap()
        }
    }
}

// helpers
impl ClassLoaderData {
    pub fn try_mem_alloc(&self, size: ByteSize, order: Ordering) -> *mut HeapWord {
        let chunk = self.current_chunk.load(order);
        if chunk.is_null() { return null_mut() }

        unsafe {
            (*chunk).bumper.par_alloc_with_size(size.into())
        }
    }

    pub fn release_new_chunk(&mut self, chunk: &mut MSChunk) {
        self.current_chunk.store(chunk, Ordering::Release);
        self.owned_chunks.push_back(chunk);
    }

    pub fn claim_new_sized_chunk(&mut self, chunk: &mut MSChunk) {
        self.owned_chunks.push_back(chunk);
    }

    pub fn drop_chunks<F: FnMut(NonNull<MSChunk>)>(&mut self, mut f: F) {
        self.current_chunk.store(null_mut(), Ordering::Relaxed);
        loop {
            match self.owned_chunks.pop_back() {
                Some(x) => unsafe { f(NonNull::new_unchecked(x)) },
                None => break
            }
        }
    }
}
