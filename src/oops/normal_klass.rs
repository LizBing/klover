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

use crate::classfile::class_loader_data::ClassLoaderData;

#[derive(Debug)]
pub struct NormalKlass {
    name: String,
    loader: Arc<ClassLoaderData>
}

impl NormalKlass {
    pub fn new(stream: Vec<u8>, loader: Arc<ClassLoaderData>) -> Result<Self, String> {
        let parsed = match cafebabe::parse_class(stream.as_slice()) {
            Ok(x) => x,

            Err(_) => return Err(String::from("bad class file stream"))
        };

        let klass = Self {
            name: parsed.this_class.to_string(),
            loader: loader
        };

        Ok(klass)
    }
}
