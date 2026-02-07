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

use crate::{classfile::class_loader_data::ClassLoaderData, oops::oop_hierarchy::OOP};

const INIT_CAPACITY_OF_CLD_GRAPH: usize = 128;

#[derive(Debug)]
pub struct ClassLoaderDataGraph {
    array: Vec<NonNull<ClassLoaderData>>,
    bs_cld: NonNull<ClassLoaderData>
}

impl ClassLoaderDataGraph {
    pub fn new() -> Self {
        let mut cld = unsafe { Box::<ClassLoaderData>::new_uninit().assume_init() };
        cld.init_bootstrap();

        Self {
            array: Vec::with_capacity(INIT_CAPACITY_OF_CLD_GRAPH),
            bs_cld: unsafe { NonNull::new_unchecked(Box::leak(cld)) }
        }
    }
}

impl ClassLoaderDataGraph {
    pub async fn register_cld(&mut self, loader: OOP) -> NonNull<ClassLoaderData> {
        assert!(!loader.is_null());

        let mut cld = unsafe { Box::<ClassLoaderData>::new_uninit().assume_init() };
        cld.init(loader).await;

        let res = unsafe { NonNull::new_unchecked(Box::leak(cld)) };
        self.array.push(res);

        res
    }

    pub fn find_cld(&self, loader: OOP) -> Option<NonNull<ClassLoaderData>> {
        if loader.is_null() {
            return Some(self.bs_cld);
        }

        for n in &self.array {
            unsafe {
                if n.as_ref().mirror.equals(loader) {
                    return Some(n.clone());
                }
            }
        }

        None
    }
}
