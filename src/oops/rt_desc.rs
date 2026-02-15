/*
 * Copyright 2026 Lei Zaakjyu
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

use std::marker::PhantomData;

use cafebabe::descriptors::{FieldDescriptor, FieldType};

use crate::{oops::{klass::KlassHandle, symbol::SymbolHandle}, runtime::universe::Universe};

pub enum PrimFieldType {
    Bool,
    Byte,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
}

pub enum RtFieldType {
    Primitive(PrimFieldType),
    Object(KlassHandle),
}

pub struct RtFieldDesc {
    pub field_type: RtFieldType,

    __: PhantomData<()>
}

impl RtFieldDesc {
    pub fn from_desc(desc: &FieldDescriptor, perm: bool) -> Self {
        let st = Universe::symbol_table();

        Self {
            dimemsion: desc.dimensions as _,
            field_type: match &desc.field_type {
                FieldType::Boolean => RtFieldType::Bool,
                FieldType::Byte => RtFieldType::Byte,
                FieldType::Char => RtFieldType::Char,
                FieldType::Short => RtFieldType::Short,
                FieldType::Integer => RtFieldType::Int,
                FieldType::Long => RtFieldType::Long,
                FieldType::Float => RtFieldType::Float,
                FieldType::Double => RtFieldType::Double,
                FieldType::Object(x) => RtFieldType::Unresolved(st.intern(x.as_bytes(), perm)),
            },

            __: PhantomData
        }
    }

    pub fn from_symbol(bytes: &[u8], perm: bool) -> Self {
        Self {
            dimemsion: 0,
            field_type: RtFieldType::Unresolved(Universe::symbol_table().intern(bytes, perm)),

            __: PhantomData
        }
    }
}
