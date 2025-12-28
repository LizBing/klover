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

use std::{cell::LazyCell, sync::LazyLock};

use dashmap::DashMap;

use crate::{gc::oop_storage::OOPStorage, oops::{klass::Klass, oop_hierarchy::OOP, weak_handle::WeakHandle}, runtime::tls::ThrdLocalStorage};

#[derive(Debug)]
pub struct ClassLoaderData {
    _mirror: WeakHandle,
    _klass_map: DashMap<String, &'static Klass<'static>>
}

impl ClassLoaderData {
    pub fn new(loader: OOP) -> Self {
        unimplemented!()
    }
}
