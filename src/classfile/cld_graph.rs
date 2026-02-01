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

use std::sync::{OnceLock, mpsc::Sender};

use crossbeam::queue::SegQueue;

use crate::classfile::class_loader_data::{CLDMsg, ClassLoaderData};

static GRAPH: OnceLock<ClassLoaderDataGraph> = OnceLock::new();

#[derive(Debug)]
pub struct ClassLoaderDataGraph {
    _queue: SegQueue<Sender<CLDMsg>>
}

impl ClassLoaderDataGraph {
    fn new() -> Self {
        Self {
            _queue: SegQueue::new()
        }
    }

    pub fn initialize() {
        GRAPH.set(Self::new()).unwrap()
    }
}

impl ClassLoaderDataGraph {
    pub fn register(cld_sender: Sender<CLDMsg>) {
        GRAPH.get().unwrap()._queue.push(cld_sender);
    }
}
