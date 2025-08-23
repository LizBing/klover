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
use dashmap::DashMap;
use once_cell::{unsync::OnceCell};
use crate::{class_data::{class_file_stream::resolve_class_name, java_classes::JavaLangClass}, metaspace::klass_allocator::{self, alloc_klass}, oops::{klass::{Klass, NormalKlass}, obj_handle::ObjHandle, oop::ObjPtr}, utils::easy_cell::EasyCell};

pub fn load_class(loader: ObjPtr, fqn: String) -> &'static Klass<'static> {
    unimplemented!()
}

pub fn load_normal_class<'a>(loader: ObjPtr, fqn: String) -> &'a NormalKlass<'a> {
}

pub fn define_class(loader: ObjPtr, fqn: String) -> &'static Klass<'static> {
    unimplemented!()
}
