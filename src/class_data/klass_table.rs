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

use std::{hash::Hash, sync::{atomic::{AtomicU64, Ordering}, Arc, Weak}};
use dashmap::DashMap;
use once_cell::sync::{Lazy, OnceCell};
use crate::{class_data::class_loader::ClassLoader, metaspace::klass_cell::KlassCell};

static LOADER_ID_GEN: AtomicU64 = AtomicU64::new(1);

fn next_loader_id() -> u64 {
    LOADER_ID_GEN.fetch_add(1, Ordering::SeqCst)
}

#[derive(Clone, Debug)]
pub struct LoaderKey {
    _id: u64,
    _loader: Weak<ClassLoader>
}

impl LoaderKey {
    pub fn new(loader: &Arc<ClassLoader>) -> Self {
        Self { _id: next_loader_id(), _loader: Arc::downgrade(loader) }
    }
}

impl LoaderKey {
    pub fn is_dead(&self) -> bool {
        self._loader.upgrade().is_none()
    }
}

impl PartialEq for LoaderKey {
    fn eq(&self, other: &Self) -> bool {
        self._id == other._id
    }
}

impl Eq for LoaderKey {}

impl Hash for LoaderKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self._id.hash(state);
    }
}

static TABLE: Lazy<DashMap<LoaderKey, DashMap<String, KlassCell>>> = Lazy::new(|| {
    DashMap::new()
});

pub fn register_loader(loader: &Arc<ClassLoader>) {
    loader.set_key(LoaderKey::new(loader));
}

pub fn get(loader_key: LoaderKey, fqn: String) -> Option<KlassCell> {
    match TABLE.get(&loader_key) {
        Some(sub_map) => {
            match sub_map.get(&fqn) {
                Some(n) => Some(n.clone()),
                None => None
            }
        }
        None => None
    }
}

pub fn put(loader_key: LoaderKey, fqn: String, klass: KlassCell) {
    let sub_map = TABLE.entry(loader_key).or_insert(DashMap::new());
    assert!(sub_map.insert(fqn, klass).is_none(), "Duplicated.");
}
