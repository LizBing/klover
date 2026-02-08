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

use std::{ptr::{NonNull, null_mut}, sync::atomic::AtomicPtr};

use crate::{gc::oop_storage_actor::CLD_STORAGE_INDEX, init_ll, metaspace::metaspace::MSChunk, oops::{klass::Klass, oop_handle::OOPHandle, oop_hierarchy::OOP}, utils::linked_list::{LinkedList, LinkedListNode}};

#[derive(Debug)]
pub struct ClassLoaderData {
    pub cld_graph_node: LinkedListNode<Self>,

    pub mirror: OOPHandle,
    klasses: LinkedList<Klass>,

    ms_chunk: AtomicPtr<MSChunk>
}

impl ClassLoaderData {
    pub async fn init(&mut self, loader: OOP) {
        *self = Self {
            cld_graph_node: LinkedListNode::new(),

            mirror: OOPHandle::with_storage(CLD_STORAGE_INDEX).await,
            klasses: LinkedList::new(),

            ms_chunk: AtomicPtr::new(null_mut())
        };
        init_ll!(&mut self.klasses, Klass, cld_node);

        self.mirror.store(loader);
    }

    pub fn init_bootstrap(&mut self) {
        *self = Self {
            cld_graph_node: LinkedListNode::new(),

            mirror: OOPHandle::new(),
            klasses: LinkedList::new(),

            ms_chunk: AtomicPtr::new(null_mut())
        };

        init_ll!(&mut self.klasses, Klass, cld_node);
    }
}

impl ClassLoaderData {
    // returns false if duplicated
    pub fn register_klass(&mut self, mut klass: NonNull<Klass>) -> bool {
        if self.klasses.iterate(|iter| -> Option<()> {
            if iter.value().name() == unsafe { klass.as_ref().name() } {
                Some(())
            } else { None }
        }).is_some() {
            return false;
        }

        unsafe { self.klasses.push_back(klass.as_mut()); }

        true
    }
}
