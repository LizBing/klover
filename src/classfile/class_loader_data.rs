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

use std::ptr::NonNull;

use crate::{gc::oop_storage_set::OOPStorageSet, init_ll, oops::{klass::Klass, oop_hierarchy::OOP, weak_handle::WeakHandle}, utils::linked_list::{LinkedList, LinkedListNode}};

#[derive(Debug)]
pub struct ClassLoaderData {
    pub cld_graph_node: LinkedListNode<Self>,

    mirror: WeakHandle,
    klasses: LinkedList<Klass>,
}

impl ClassLoaderData {
    pub fn init(&mut self, loader: OOP) {
        *self = Self {
            cld_graph_node: LinkedListNode::new(),

            mirror: WeakHandle::new(OOPStorageSet::cld_weak_storage()),
            klasses: LinkedList::new()
        };

        init_ll!(&mut self.klasses, Klass, cld_node);
    }
}

impl ClassLoaderData {
    // returns false if duplicated
    pub fn register_klass(&mut self, klass: NonNull<Klass>) -> bool {
        if self.klasses.iterate(|iter| -> Option<()> {
            if iter.value().name() == unsafe { klass.as_ref().name() } {
                Some(())
            } else { None }
        }).is_some() {
            return false;
        }

        unsafe { self.klasses.push_back(klass.as_ref()); }

        true
    }
}
