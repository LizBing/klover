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

use std::{mem::offset_of, ptr::null};

use crate::{gc::barrier_set::AccessBarrier, oops::{access::AccessAPI, mark_word::AtomicMarkWord, obj_desc::{ArrayObjDesc, ObjDesc}}};

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct OOP(*const ObjDesc);

impl OOP {
    pub fn is_null(self) -> bool {
        self.0 == null()
    }

    pub fn mark_word<'a>(base: Self) -> &'a AtomicMarkWord {
        unsafe { 
            &*(base.0.byte_add(ObjDesc::mark_word_offset()) as *const _)
        }
    }

    pub fn null() -> OOP {
        OOP(null())
    }
}

pub struct ArrayOOP;

impl ArrayOOP {
    pub fn length(base: OOP) -> usize {
        unsafe {
            *(base.0.byte_add(ArrayObjDesc::length_offset()) as *const u32) as usize
        }
    }

    pub fn set_length(base: OOP, length: i32) {
        unsafe {
            *(base.0.byte_add(ArrayObjDesc::length_offset()) as *mut _) = length
        }
    }
}
