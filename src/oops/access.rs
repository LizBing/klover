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

use std::marker::PhantomData;

use bitflags::bitflags;

use crate::{gc::barrier_set::AccessBarrier, oops::oop_hierarchy::{NarrowOOP, OOP}};

const DECORATOR_NONE: u32 = 0;

// Internals
// const DECORATOR_INTERNAL_VALUE_IS_OOP:  u32 = 1u32 << 1;

// Reference strength
const DECORATOR_ON_STRONG_REF:      u32 = 1u32 << 2;
const DECORATOR_ON_WEAK_REF:        u32 = 1u32 << 3;
const DECORATOR_ON_PHANTOM_REF:     u32 = 1u32 << 4;
const DECORATOR_ON_UNKNOWN_REF:     u32 = 1u32 << 5;
const REF_STRENGTH_DECORATORS_MASK: u32 =
    DECORATOR_ON_STRONG_REF     |
    DECORATOR_ON_WEAK_REF       |
    DECORATOR_ON_PHANTOM_REF    |
    DECORATOR_ON_UNKNOWN_REF    ;

// Memory order
const DECORATOR_MO_UNORDERED:   u32 = 1u32 << 6;
const DECORATOR_MO_RELAXED:     u32 = 1u32 << 7;
const DECORATOR_MO_ACQUIRE:     u32 = 1u32 << 8;
const DECORATOR_MO_RELEASE:     u32 = 1u32 << 9;
const DECORATOR_MO_SEQ_CST:     u32 = 1u32 << 10;
const MO_DECORATORS_MASK:       u32 =
    DECORATOR_MO_UNORDERED |
    DECORATOR_MO_RELAXED   |
    DECORATOR_MO_ACQUIRE   |
    DECORATOR_MO_RELEASE   |
    DECORATOR_MO_SEQ_CST   ;

// Location
const DECORATOR_IN_HEAP:        u32 = 1u32 << 11;
const DECORATOR_IN_NATIVE:      u32 = 1u32 << 12;
const DECORATOR_IN_NMETHOD:     u32 = 1u32 << 13;
const LOCATION_DECORATORS_MASK: u32 =
    DECORATOR_IN_HEAP       |
    DECORATOR_IN_NATIVE     |
    DECORATOR_IN_NMETHOD    ;

// Barrier strength
const DECORATOR_AS_RAW:                 u32 = 1u32 << 14;
const DECORATOR_AS_NO_KEEP_ALIVE:       u32 = 1u32 << 15;
const DECORATOR_AS_NORMAL:              u32 = 1u32 << 16;
const BARRIER_STRENGTH_DECORATORS_MASK: u32 =
    DECORATOR_AS_RAW            |
    DECORATOR_AS_NO_KEEP_ALIVE  |
    DECORATOR_AS_NORMAL         ;

// Boolean flags
const DECORATOR_IS_ARRAY: u32 = 1u32 << 17;
// const DECORATOR_IS_NOT_NULL: u32 = 1u32 << 18;
const DECORATOR_IS_DEST_UNINITIALIZED: u32 = 1u32 << 19;

const DECORATOR_LAST: u32 = 1u32 << 20;

bitflags! {
    pub struct DecoratorSet: u32 {
        const NONE = DECORATOR_NONE;
        // const INTERNAL_VALUE_IS_OOP = DECORATOR_INTERNAL_VALUE_IS_OOP;

        const ON_STRONG_REF = DECORATOR_ON_STRONG_REF;
        const ON_WEAK_REF = DECORATOR_ON_WEAK_REF;
        const ON_PHANTOM_REF = DECORATOR_ON_PHANTOM_REF;

        const MO_UNORDERED = DECORATOR_MO_UNORDERED;
        const MO_RELAXED = DECORATOR_MO_RELAXED;
        const MO_ACQUIRE = DECORATOR_MO_ACQUIRE;
        const MO_RELEASE = DECORATOR_MO_RELEASE;
        const MO_SEQ_CST = DECORATOR_MO_SEQ_CST;

        const IN_HEAP = DECORATOR_IN_HEAP;
        const IN_NATIVE = DECORATOR_IN_NATIVE;
        const IN_NMETHOD = DECORATOR_IN_NMETHOD;

        const AS_RAW = DECORATOR_AS_RAW;
        const AS_NO_KEEP_ALIVE = DECORATOR_AS_NO_KEEP_ALIVE;
        const AS_NORMAL = DECORATOR_AS_NORMAL;

        const IS_ARRAY = DECORATOR_IS_ARRAY;
        // const IS_NOT_NULL = DECORATOR_IS_NOT_NULL;
        const IS_DEST_UNINITIALIZED = DECORATOR_IS_DEST_UNINITIALIZED;
    }
}

pub struct Access<const D: u32>;

impl<const D: u32> Access<D> {
    pub fn oop_load<P>(addr: *const P) -> NarrowOOP {
        unimplemented!()
    }
    
    pub fn oop_load_at(base: NarrowOOP, byte_offs: usize) -> NarrowOOP {
        unimplemented!()
    }

    pub fn load_at<T: Copy>(base: NarrowOOP,  byte_offs: usize) -> T {
        unimplemented!()
    }

    pub fn oop_store<P>(addr: *const P, n: NarrowOOP) {
        unimplemented!()
    }

    pub fn oop_store_at(base: NarrowOOP, byte_offs: usize, oop: NarrowOOP) {
        unimplemented!()
    }

    pub fn store_at<T>(base: NarrowOOP, byte_offs: usize, n: T) {
        unimplemented!()
    }
}

