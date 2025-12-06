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

use crate::{gc::barrier_set::AccessBarrier, oops::{access::{Access, DecoratorSet}, mark_word::MarkWord, obj_desc::{ArrayObjDesc, ObjDesc}}};

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct OOP(*const ObjDesc);

impl OOP {
    const fn decorators(d: u32) -> DecoratorSet {
        DecoratorSet::from_bits_truncate(d)
    }

    pub fn is_null(self) -> bool {
        self.0 == null()
    }

    pub fn mark_word<'a, const D: u32>(self) -> MarkWord {
        if Self::decorators(D).contains(DecoratorSet::IN_HEAP) {
            AccessBarrier::<D>::load_in_heap_at(self, ObjDesc::mark_word_offset())
        } else {
            AccessBarrier::<D>::load_not_in_heap_at(self, ObjDesc::mark_word_offset())
        }
    }

    pub fn null() -> OOP {
        OOP(null())
    }
}

pub struct ArrayOOP;
impl ArrayOOP {
    const fn decorators(d: u32) -> DecoratorSet {
        DecoratorSet::from_bits_truncate(d)
    }

    pub fn length<const D: u32>(base: OOP) -> usize {
        if Self::decorators(D).contains(DecoratorSet::IN_HEAP) {
            AccessBarrier::<D>::load_in_heap_at::<i32>(base, ArrayObjDesc::length_offset()) as _
        } else {
            AccessBarrier::<D>::load_not_in_heap_at::<i32>(base, ArrayObjDesc::length_offset()) as _
        }
    }

    pub fn set_length<const D: u32>(base: OOP, length: i32) {
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
