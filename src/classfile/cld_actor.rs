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

use std::ptr::NonNull;
use tokio::sync::mpsc;

use crate::{classfile::{class_loader_data::{CLDHandle, ClassLoaderData}, cld_graph::ClassLoaderDataGraph}, oops::{klass::{Klass, KlassHandle}, oop_hierarchy::OOP}};

pub enum CLDMsg {
    RegisterCLD { loader: OOP, reply_tx: mpsc::Sender<NonNull<ClassLoaderData>> },

    RegisterKlass { loader: CLDHandle, klass: KlassHandle, reply_tx: mpsc::Sender<bool> },

    FindCLD { loader: OOP, reply_tx: mpsc::Sender<Option<NonNull<ClassLoaderData>>> },

    Shutdown
}

unsafe impl Send for CLDMsg {}

pub struct CLDActor {
    rx: mpsc::UnboundedReceiver<CLDMsg>,
    graph: ClassLoaderDataGraph
}

unsafe impl Send for CLDActor {}

impl CLDActor {
    pub fn new(rx: mpsc::UnboundedReceiver<CLDMsg>) -> CLDActor {
        Self {
            rx: rx,
            graph: ClassLoaderDataGraph::new()
        }
    }
}

impl CLDActor {
    pub async fn run(mut self) {
        loop {
            match self.rx.recv().await.unwrap() {
                CLDMsg::RegisterCLD { loader, reply_tx } => {
                    let cld = self.graph.register_cld(loader);
                    reply_tx.blocking_send(cld).unwrap();
                }

                CLDMsg::RegisterKlass { mut loader, klass, reply_tx } => {
                    let res = loader.register_klass(klass);
                    reply_tx.blocking_send(res).unwrap();
                }

                CLDMsg::FindCLD { loader, reply_tx } => {
                    let res = self.graph.find_cld(loader);
                    reply_tx.blocking_send(res).unwrap();
                }

                CLDMsg::Shutdown => break
            }
        }
    }
}
