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

use cafebabe::ClassFile;

use crate::oops::{array_klass::ArrayKlass, normal_klass::NormalKlass, prim_klass::PrimKlass};

#[derive(Debug)]
pub enum Klass<'a> {
    Normal(NormalKlass<'a>),
    Primitive(PrimKlass),
    ArrayKlass(ArrayKlass<'a>),
}

unsafe impl Sync for Klass<'_> {}

impl<'a> Klass<'a> {
    pub fn as_normal(&self) -> &NormalKlass<'a> {
        match self {
            Self::Normal(x) => x,
            _ => unreachable!()
        }
    }

    pub fn as_prim(&self) -> &PrimKlass {
        match self {
            Self::Primitive(x) => x,
            _ => unreachable!()
        }
    }

    pub fn as_array_klass(&self) -> &ArrayKlass<'a> {
        match self {
            Self::ArrayKlass(x) => x,
            _ => unreachable!()
        }
    }
}

impl Klass<'_> {
    pub fn word_size_of_instance(&self) -> usize {
        unimplemented!()
    }
}
