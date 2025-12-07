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

use std::ptr::null;

use crate::oops::{mark_word::MarkWord, obj_desc::ObjDesc};

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct OOP(*const ObjDesc);

impl OOP {
    pub fn null() -> OOP {
        Self(null())
    }
}

impl OOP {
    pub fn is_null(self) -> bool {
        self.0 == null()
    }

    pub fn mark_word(self) -> MarkWord {
        unimplemented!()
    }

    pub fn set_mark_word(self, value: MarkWord) {
        unimplemented!()
    }

    pub fn cas_mark_word(self, exp: MarkWord, des: MarkWord) -> bool {
        unimplemented!()
    }
}

pub struct ArrayOOP;
impl ArrayOOP {
    pub fn length(base: OOP) -> i32 {
        unimplemented!()
    }

    // once
    pub fn set_length(base: OOP, length: i32) {
        unimplemented!()
    }

    // in bytes
    pub fn cal_offset<T>(index: i32) -> usize {
        unimplemented!()
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct NarrowOOP(u32);

impl NarrowOOP {
    pub fn null() -> Self {
        Self(0)
    }

    pub fn is_null(self) -> bool {
        self.0 == 0
    }

    pub fn encode(n: OOP) -> Self {
        unimplemented!()
    }

    pub fn decode(self) -> OOP {
        unimplemented!()
    }
}
