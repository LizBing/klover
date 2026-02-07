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

// unsafe impl Send for OOP {}

impl OOP {
    pub fn null() -> OOP {
        Self(null())
    }
    
    pub fn is_null(self) -> bool {
        self.0 == null()
    }
}

impl OOP {
    pub fn raw(self) -> *const ObjDesc {
        self.0
    }
}

impl OOP {
    pub fn mark_word<const D: u32>(self) -> MarkWord {
        unimplemented!()
    }

    pub fn set_mark_word<const D: u32>(self, value: MarkWord) {
        unimplemented!()
    }

    pub fn cas_mark_word<const D: u32>(self, exp: MarkWord, des: MarkWord) -> bool {
        unimplemented!()
    }

    pub fn get_field<const D: u32, T: Copy>(self, offset: usize) -> T {
        unimplemented!()
    }

    pub fn put_field<const D: u32, T>(self, offset: usize, value: T) {
        unimplemented!()
    }

    pub fn get_oop_field<const D: u32>(self, offset: usize) -> OOP {
        unimplemented!()
    }

    pub fn put_oop_field<const D: u32>(self, offset: usize, value: OOP) {
        unimplemented!()
    }
}

pub struct ArrayOOP;
impl ArrayOOP {
    pub fn length<const D: u32>(base: OOP) -> i32 {
        unimplemented!()
    }

    // once
    pub fn set_length<const D: u32>(base: OOP, length: i32) {
        unimplemented!()
    }

    pub fn get<const D: u32, T: Copy>(base: OOP, index: i32) -> T {
        unimplemented!()
    }

    pub fn put<const D: u32, T>(base: OOP, index: i32, value: T) {
        unimplemented!()
    }

    pub fn get_oop<const D: u32>(base: OOP, index: i32) -> OOP {
        unimplemented!()
    }

    pub fn put_oop<const D: u32>(base: OOP, index: i32, value: OOP) {
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
