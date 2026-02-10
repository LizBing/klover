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

use std::ptr::NonNull;

use crate::{classfile::class_loader_data::ClassLoaderData, oops::{array_klass::ArrayKlass, normal_klass::NormalKlass, prim_klass::PrimKlass, symbol::Symbol}, utils::linked_list::LinkedListNode};

#[derive(Debug)]
pub enum KlassData<'a> {
    Normal(NormalKlass<'a>),
    Prim(PrimKlass),
    ArrayKlass(ArrayKlass)
}

#[derive(Debug)]
pub struct Klass {
    pub cld_node: LinkedListNode<Self>,
    klass_data: KlassData<'static>,
}

impl Klass {
    pub fn name(&self) -> &Symbol {
        unimplemented!()
    }

    pub fn cld(&self) -> &ClassLoaderData {
        unimplemented!()
    }

    pub fn unit_word_size(&self) -> usize {
        unimplemented!()
    }
}
