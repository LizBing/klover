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

use cafebabe::{attributes::{AttributeData, AttributeInfo, CodeData}, bytecode::Opcode, MethodInfo};

use crate::{code::cp_cache::{self, ConstantPoolCache}, oops::klass::{Klass, NormalKlass}};

pub struct Method<'a> {
    _info: &'a MethodInfo<'a>,
    _code_data: Option<&'a CodeData<'a>>,

    _klass: &'static NormalKlass<'static>,
    _cp_cache: ConstantPoolCache<'a>
}

impl<'a> Method<'a> {
    pub fn new(info: &'a MethodInfo, klass: &'static NormalKlass) -> Self {
        let mut code_data = None;
        for n in &info.attributes {
            match &n.data {
                AttributeData::Code(cd) => {
                    code_data = Some(cd);
                    break;
                }

                _ => continue
            }
        }

        Self {
            _info: info,
            _code_data: code_data,
            _klass: klass,
            _cp_cache: ConstantPoolCache::new(klass.cp_entries())
        }
    }
}

impl<'a> Method<'a> {
    pub fn code_data(&self) -> Option<&CodeData> {
        self._code_data
    }

    pub fn klass(&self) -> &'static NormalKlass {
        self._klass
    }

    pub fn cp_cache(&self) -> &'a ConstantPoolCache {
        &self._cp_cache
    }
}

impl Method<'_> {
    pub fn reflect_cp_index(&self, code_offs: usize) -> usize {
        let code = self._code_data.unwrap().code;

        let indexbyte1 = code[code_offs + 1] as u16;
        let indexbyte2 = code[code_offs + 2] as u16;

        let index = (indexbyte1 << 8) | indexbyte2;

        index as usize
    }
}

