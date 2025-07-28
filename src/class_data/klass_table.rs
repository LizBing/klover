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
use once_cell::sync::OnceCell;
use dashmap::DashMap;
use crate::{metaspace::klass_cell::KlassCell};

type KlassTable = DashMap<String, KlassCell>;
static KLASS_TABLE: OnceCell<KlassTable> = OnceCell::new();

fn table() -> &'static KlassTable {
    KLASS_TABLE.get().unwrap()
}

pub fn get(fqn: String) -> Option<KlassCell> {
    match table().get(&fqn) {
        Some(handle) => { Some(handle.clone()) },
        None => None
    }
}

pub fn put(klass: KlassCell) {
    unimplemented!()
}
