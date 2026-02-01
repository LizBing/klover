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

use cafebabe::{attributes::CodeData};

use crate::oops::klass::Klass;

#[derive(Debug)]
pub struct Method {
    _klass: NonNull<Klass>
}

impl Method {
    pub fn code_data(&self) -> Option<&CodeData<'_>> {
        unimplemented!()
    }

    pub fn klass(&self) -> NonNull<Klass> {
        self._klass
    }
}
