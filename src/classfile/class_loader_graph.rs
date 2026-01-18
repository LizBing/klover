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

use std::{ptr::null, sync::{Arc, LazyLock, atomic::{AtomicU32, Ordering}}};

use dashmap::DashMap;

use crate::{classfile::{class_loader_data::ClassLoaderData, java_classes::JavaLangClassLoader}, gc::oop_storage::OOPStorage, oops::{access::{Access, DECORATOR_MO_VOLATILE}, oop_hierarchy::OOP, weak_handle::WeakHandle}, runtime::universe::Universe};

static KEY_ALLOCATOR: AtomicU32 = AtomicU32::new(1);
static BOOTSTRAP_CLD: LazyLock<ClassLoaderData> = LazyLock::new(ClassLoaderData::new_bootstrap);
static CLD_MAP: LazyLock<DashMap<u32, Arc<ClassLoaderData>>> = LazyLock::new(DashMap::new);

struct ClassLoaderGraph;
impl<'a> ClassLoaderGraph {
    pub fn get_cld<const D: u32>(loader: OOP) -> Arc<ClassLoaderData> {
        unimplemented!()
    }

    pub fn get_bootstrap_cld() -> &'static ClassLoaderData {
        &BOOTSTRAP_CLD
    }
}
