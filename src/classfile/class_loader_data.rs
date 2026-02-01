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

use std::{ptr::{NonNull, null}, sync::mpsc::{self, Sender}, thread};

use crate::{create_ll, gc::oop_storage_set::OOPStorageSet, oops::{klass::Klass, oop_hierarchy::OOP, weak_handle::WeakHandle}, utils::linked_list::LinkedList};

#[derive(Debug)]
pub struct ClassLoaderData {
    _mirror: WeakHandle,
    _klasses: LinkedList<Klass>,
}

impl ClassLoaderData {
    fn new(loader: OOP) -> Self {
        Self {
            _mirror: WeakHandle::new(OOPStorageSet::cld_weak_storage()),
            _klasses: create_ll!(Klass, cld_node)
        }
    }
}

impl ClassLoaderData {
    // returns false if duplicated
    fn register(&mut self, klass: NonNull<Klass>) -> bool {
        unsafe {
            if self._klasses.iterate_reversed(|n| -> _ {
                if n.name() == klass.as_ref().name() {
                    Some(())
                } else { None }
            }).is_some() {
                return false;
            }

            self._klasses.push_back(klass.as_ref());

            true
        }
    }
}

pub enum CLDMsg {
    RegisterKlass { klass: NonNull<Klass>, reply: Sender<()> },
    Shutdown
}

unsafe impl Send for CLDMsg {}
unsafe impl Sync for CLDMsg {}

impl ClassLoaderData {
    pub fn spawn_actor(loader: OOP) -> Sender<CLDMsg> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut cld = Self::new(loader);

            loop {
                match rx.recv() {
                    Ok(CLDMsg::RegisterKlass { klass, reply }) => {
                        cld.register(klass);
                        reply.send(());
                    }

                    Ok(CLDMsg::Shutdown) | Err(_) => break
                }
            }
        });

        tx
    }
}
