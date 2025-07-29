use std::borrow::Cow;

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
use cafebabe::{attributes::{AttributeData, AttributeInfo, CodeData}, bytecode::Opcode};

use crate::code::cp_cache::ConstantPoolCache;

pub struct Method<'a> {
    _name: Cow<'a, str>,
    _code_data: &'a CodeData<'a>,

    pub cp_cache: ConstantPoolCache
}

impl Method<'_> {
    pub fn code_data(&self) -> &CodeData {
        self._code_data
    }

    pub fn opcodes(&self) -> &Vec<(usize, Opcode)> {
        &self._code_data.bytecode.as_ref().unwrap().opcodes
    }
}

