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

    // atomic
    pub fn mark_word(self) -> MarkWord {
        unimplemented!()
    }

    // atomic
    pub fn set_mark_word(self, value: MarkWord) {
        unimplemented!()
    }

    pub fn cas_mark_word(self, exp: MarkWord, des: MarkWord) -> bool {
        unimplemented!()
    }

    // raw access
    pub fn get_field<const VOLATILE: bool, T: Copy>(self, offset: usize) -> T {
        unimplemented!()
    }

    pub fn put_field<const VOLATILE: bool, T>(self, offset: usize, value: T) {
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

    // raw access
    pub fn get<const VOLATILE: bool, T: Copy>(base: OOP, index: i32) -> T {
        unimplemented!()
    }

    pub fn put<const VOLATILE: bool, T>(base: OOP, index: i32, value: T) {
        unimplemented!()
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct NarrowOOP(u32);

impl NarrowOOP {
    pub fn encode(n: OOP) -> Self {
        unimplemented!()
    }

    pub fn decode(self) -> OOP {
        unimplemented!()
    }
}
