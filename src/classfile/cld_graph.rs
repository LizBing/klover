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

use std::{ptr::NonNull, sync::Arc};

use crate::{classfile::class_loader_data::ClassLoaderData, oops::oop_hierarchy::OOP};

const INIT_CAPACITY_OF_CLD_GRAPH: usize = 128;

#[derive(Debug)]
pub struct ClassLoaderDataGraph {
    array: Vec<Arc<ClassLoaderData>>
}

impl ClassLoaderDataGraph {
    pub fn new() -> Self {
        Self {
            array: Vec::with_capacity(INIT_CAPACITY_OF_CLD_GRAPH)
        }
    }
}

impl ClassLoaderDataGraph {
    pub fn register_cld(&mut self, loader: OOP) -> Arc<ClassLoaderData> {
        unsafe {
            let mut uninit = Arc::<ClassLoaderData>::new_uninit();
            Arc::get_mut(&mut uninit).unwrap().assume_init_mut().init(loader);

            let cld = uninit.assume_init();
            self.array.push(cld.clone());

            cld
        }
    }
}
