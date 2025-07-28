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
use std::borrow::Cow;
use std::cell::OnceCell;
use std::ptr::null_mut;
use cafebabe::descriptors::ClassName;
use dashmap::DashMap;
use crate::metaspace::klass_allocator::KlassMemSpace;
use crate::oops::klass::{Klass, KlassHandle};

static KLASS_TABLE: OnceCell<DashMap<String, KlassHandle>> = OnceCell::new();

pub fn get(fqn: String) -> Option<KlassHandle> {
    let table = KLASS_TABLE.get().unwrap();

    match table.get(&fqn) {
        Some(handle) => Some(*handle),
        None => None
    }
}

pub fn put(klass: KlassHandle) {
    unimplemented!()
}
