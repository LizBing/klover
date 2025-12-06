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

use std::mem::offset_of;

use crate::oops::mark_word::MarkWord;

#[repr(C)]
pub struct ObjDesc {
    _mw: MarkWord
}

impl ObjDesc {
    pub const fn mark_word_offset() -> usize {
        offset_of!(Self, _mw)
    }

    pub const fn data_offset() -> usize {
        size_of::<Self>()
    }
}

#[repr(C)]
pub struct ArrayObjDesc {
    _super: ObjDesc,
    _len: i32
}

impl ArrayObjDesc {
    pub const fn data_offset() -> usize {
        size_of::<Self>()
    }

    pub const fn length_offset() -> usize {
        offset_of!(Self, _len)
    }
}
