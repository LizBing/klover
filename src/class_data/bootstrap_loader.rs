use std::sync::{Arc, RwLock};

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
use cafebabe::descriptors::ClassName;
use dashmap::{DashMap, Entry};
use once_cell::sync::Lazy;
use crate::{class_data::{class_loader::ClassLoader, java_classes::JavaLangClass}, metaspace::{klass_allocator::alloc_klass, klass_cell::KlassCell}, oops::klass::Klass, runtime::tls};

static TABLE: Lazy<DashMap<String, KlassCell>> = Lazy::new(|| { DashMap::new() });

// Returns None for LinkageError.
pub fn define_class(fqn: String, buf: Vec<u8>) -> Option<KlassCell> {
    let klass = ClassLoader::define_class_helper(None, buf);

    match TABLE.entry(fqn) {
        Entry::Vacant(entry) => {
            entry.insert(klass.clone());
            Some(klass)
        },
        _ => None,
    }
}

pub fn load_class(name: String) -> Option<KlassCell> {
    unimplemented!()
}
