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

use cafebabe::{MethodAccessFlags, MethodInfo, attributes::{AttributeInfo, CodeData}, descriptors::MethodDescriptor};

use crate::{oops::{klass::{Klass, KlassHandle}, symbol::SymbolHandle}, runtime::universe::Universe};

#[derive(Debug)]
pub struct Method<'a> {
    name: SymbolHandle,
    acc_flags: MethodAccessFlags,
    desc: MethodDescriptor<'a>,
    attrs: Vec<AttributeInfo<'a>>,

    klass: KlassHandle,
}

impl<'a> Method<'a> {
    pub fn new<'b: 'a>(minfo: &'b MethodInfo, klass: KlassHandle) -> Self {
        let cld = klass.cld();
        let perm_sym = cld.is_bootstrap_cld();

        let mut attrs = Vec::new();

        Self {
            name: Universe::symbol_table().intern(minfo.name.as_bytes(), perm_sym),
            acc_flags: minfo.access_flags,
            desc: minfo.descriptor.clone(),
            attrs: attrs,
            klass: klass
        }
    }
}

impl Method {
    pub fn code_data(&self) -> Option<&CodeData<'_>> {
        unimplemented!()
    }

    pub fn klass(&self) -> NonNull<Klass> {
        unimplemented!()
    }
}
