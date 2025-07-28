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

use std::ptr::null_mut;
use cafebabe::descriptors::ClassName;
use crate::class_data::class_file_stream::resolve_class_name;
use crate::class_data::klass_table;
use crate::oops::klass::{Klass, KlassHandle};
use crate::oops::obj_handle::ObjHandle;
use crate::runtime::tls;

pub struct ClassLoader {}

impl ClassLoader {
    pub fn load_class(&self, name: ClassName) -> Option<KlassHandle> {
        unimplemented!()
    }
}

fn foo() {
    let cf = cafebabe::parse_class(String::new().as_bytes()).unwrap();
}
