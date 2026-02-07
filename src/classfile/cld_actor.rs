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

use crate::{classfile::{class_loader_data::ClassLoaderData, cld_graph::ClassLoaderDataGraph}, oops::{klass::Klass, oop_hierarchy::OOP}};

pub enum CLDMsg {
    RegisterCLD { loader: OOP, reply_tx: mpsc::Sender<NonNull<ClassLoaderData>> },

    RegisterKlass { loader: NonNull<ClassLoaderData>, klass: NonNull<Klass>, reply_tx: mpsc::Sender<bool> },

    GetCLD { loader: OOP, reply_tx: mpsc::Sender<NonNull<ClassLoaderData>> },

    Shutdown
}

unsafe impl Send for CLDMsg {}

pub struct CLDActor {
    rx: mpsc::UnboundedReceiver<CLDMsg>,
    graph: ClassLoaderDataGraph
}

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
                    let cld = self.graph.register_cld(loader).await;
                    reply_tx.send(cld).await.unwrap();
                }

                CLDMsg::RegisterKlass { mut loader, klass, reply_tx } => {
                    let res = unsafe { loader.as_mut().register_klass(klass) };
                    reply_tx.send(res).await.unwrap();
                }

                CLDMsg::GetCLD { loader, reply_tx } => {}

                CLDMsg::Shutdown => break
            }
        }
    }
}
