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

use std::{ptr::NonNull, sync::Arc};

use crate::{classfile::class_loader_data::ClassLoaderData, metaspace::klass_space::KlassSpace, oops::{klass::Klass, normal_klass::NormalKlass}};

pub struct ClassLoader;

impl ClassLoader {
    pub fn define_normal_class(loader: Option<Arc<ClassLoaderData>>, stream: Vec<u8>) -> Result<NonNull<Klass>, String> {
        let klass = Klass::Normal(NormalKlass::new(stream)?);
        let res = KlassSpace::space().par_alloc(klass);

        // todo: register to CLD.

        Ok(res)
    }

    pub fn load_class(loader: Option<Arc<ClassLoaderData>>, name: String) -> Result<NonNull<Klass>, String> {
        unimplemented!()
    }

    pub fn find_class(loader: Option<Arc<ClassLoaderData>>, name: String) -> Result<NonNull<Klass>, String> {
        unimplemented!()
    }
}
