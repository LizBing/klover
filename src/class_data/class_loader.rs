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

use std::sync::Arc;

use cafebabe::descriptors::ClassName;
use once_cell::{unsync::OnceCell};
use crate::{class_data::{java_classes::JavaLangClass, klass_table::{self, LoaderKey}}, metaspace::{klass_allocator::alloc_klass, klass_cell::KlassCell}};


#[derive(Debug)]
pub struct ClassLoader {
    _key: OnceCell<LoaderKey>,
}

unsafe impl Sync for ClassLoader {}

impl ClassLoader {
    pub fn key(&self) -> LoaderKey {
        self._key.get().unwrap().clone()
    }

    pub fn set_key(&self, key: LoaderKey) {
        self._key.set(key).unwrap();
    }
}

impl ClassLoader {
    pub fn define_class_helper(loader: Option<Arc<ClassLoader>>, buf: Vec<u8>) -> KlassCell {
        let klass = alloc_klass();

        klass.get_mut().init_normal(loader, buf);
        klass.get_mut().set_mirror(JavaLangClass::new_instance(klass.clone()));

        klass
    }

    pub fn load_class(&self, name: String) -> Option<KlassCell> {
        unimplemented!()
    }

    pub fn define_class(loader: Arc<ClassLoader>, fqn: String, buf: Vec<u8>) -> KlassCell {
        let klass = Self::define_class_helper(Some(loader.clone()), buf);
        klass_table::put(loader.key(), fqn, klass.clone());

        klass
    }
}
