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

use std::borrow::Cow;

use cafebabe::{attributes::CodeData, MethodInfo};

use crate::oops::klass::Klass;

#[derive(Debug)]
pub struct Method<'a> {
    _klass: &'a Klass<'a>,
    _info: &'a MethodInfo<'a>,
    _code_data: Option<&'a CodeData<'a>>,
}

impl Method<'_> {
    pub fn code_data(&self) -> Option<&CodeData> {
        self._code_data
    }
}
