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

use crate::oops::obj_desc::{ArrayObjDesc, ObjDesc};

#[repr(transparent)]
pub struct OOP(*const ObjDesc);

#[repr(transparent)]
pub struct ArrayOOP(*const ObjDesc);

impl From<OOP> for ArrayOOP {
    fn from(value: OOP) -> Self {
        Self(value.0 as _)
    }
}

impl From<ArrayOOP> for OOP {
    fn from(value: ArrayOOP) -> Self {
        Self(value.0 as _)
    }
}
