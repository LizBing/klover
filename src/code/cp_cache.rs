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

use std::sync::RwLock;

#[derive(Clone)]
pub enum ConstantPoolCacheEntry {
    None,
}

pub struct ConstantPoolCache {
    _entries: RwLock<Vec<ConstantPoolCacheEntry>>
}

impl ConstantPoolCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            _entries: RwLock::new(Vec::with_capacity(capacity))
        }
    }
}

impl ConstantPoolCache {
    pub fn acquire(&self, index: usize) -> ConstantPoolCacheEntry {
        let guard = self._entries.read().unwrap();
        let res = guard[index].clone();

        res
    }

    pub fn resolve(&self, index: usize, data: ConstantPoolCacheEntry) {
        let mut guard = self._entries.write().unwrap();
        guard[index] = data;
    }
}
