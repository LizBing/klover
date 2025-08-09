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

use crate::{oops::klass::Klass, utils::easy_cell::EasyCell};

static LOADER_ID_GEN: AtomicU64 = AtomicU64::new(1);

fn next_loader_id() -> u64 {
    LOADER_ID_GEN.fetch_add(1, Ordering::SeqCst)
}

#[derive(Clone, Debug)]
pub struct LoaderKey {
    _id: u64,
}

impl LoaderKey {
    pub fn new() -> Self {
        unimplemented!()
    }
}

impl LoaderKey {
    pub fn is_dead(&self) -> bool {
        unimplemented!()
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

static TABLE: Lazy<DashMap<Option<LoaderKey>, DashMap<String, EasyCell<Klass>>>> = Lazy::new(|| {
    DashMap::new()
});

pub fn register_loader() {
    unimplemented!()
}

pub fn get(loader_key: Option<LoaderKey>, fqn: String) -> Option<&'static Klass<'static>> {
    unimplemented!()
}

pub fn put(loader_key: Option<LoaderKey>, fqn: String, klass: &Klass) {
    unimplemented!()
}
