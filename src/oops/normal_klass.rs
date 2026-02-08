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

use std::{cell::OnceCell, ptr::NonNull, sync::Arc};

use cafebabe::{ClassAccessFlags, ClassFile};

use crate::classfile::class_loader_data::ClassLoaderData;

#[derive(Debug)]
pub struct NormalKlass<'a> {
    raw_bytes: Vec<u8>,
    parsed: OnceCell<ClassFile<'a>>,
    loader: NonNull<ClassLoaderData>,
}

impl<'a> NormalKlass<'a> {
    pub fn new(stream: Vec<u8>, loader: NonNull<ClassLoaderData>) -> Self {
        Self {
            raw_bytes: stream,
            parsed: OnceCell::new(),
            loader: loader
        }
    }

    pub fn init(&'a self) -> bool {
        let parsed = match cafebabe::parse_class(&self.raw_bytes) {
            Ok(x) => x,
            Err(_) => return false
        };
        self.parsed.set(parsed).unwrap();

        true
    }
}
