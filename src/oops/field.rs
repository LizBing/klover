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

use cafebabe::{descriptors::FieldType, ClassFile, FieldInfo};

use crate::{runtime::{jvalue::Jvalue, runtime_globals::{self, RUNTIME_GLOBALS}}, utils::global_defs::{address, naddr}};

#[derive(Debug)]
pub struct Field<'a> {
    _info: &'a FieldInfo<'a>,
    _offs: usize,   // offset from the obj header(ObjPtr)
    _static: Option<Jvalue>
}

impl<'a> Field<'a> {
    pub fn new_non_static(info: &'a FieldInfo, offs: usize) -> Self {
        Self {
            _info: info,
            _offs: offs,
            _static: None
        }
    }
}

impl Field<'_> {
    // the offset from ObjPtr
    pub fn offset(&self) -> usize {
        self._offs
    }
}
