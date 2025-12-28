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

use cafebabe::{parse_class, ClassFile};

use crate::{classfile::{class_loader::ClassLoader, class_loader_data::ClassLoaderData}, oops::{klass::Klass, oop_handle::OOPHandle, oop_hierarchy::OOP, weak_handle::WeakHandle}};

#[derive(Debug)]
pub struct NormalKlass<'a> {
    _stream: Vec<u8>,
    _cf: Option<ClassFile<'a>>,

    _super: Option<&'a Klass<'static>>,

    _mirror: OOPHandle,
    _loader: Option<Arc<ClassLoaderData>>,
}

impl<'a> NormalKlass<'a> {
    pub fn init(&'a mut self, loader: Option<Arc<ClassLoaderData>>, stream: Vec<u8>) -> Result<(), String> {
        *self = Self {
            _stream: stream,
            _cf: None,
            _super: None,
            _mirror: OOPHandle::new(),
            _loader: loader.clone()
        };

        self._cf = Some(cafebabe::parse_class(&self._stream).unwrap());
        self._super = if let Some(x) = unsafe {
            self._cf.as_ref().unwrap_unchecked().super_class.as_ref()
        } {
            Some(ClassLoader::find_class(loader.as_ref(), x.to_string())?)
        } else {
            None
        };

        Ok(())
    }
}
