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

use crate::{gc::barrier_set::AccessBarrier, oops::oop_hierarchy::OOP};

pub const DECORATOR_NONE: u32 = 0;
pub const DECORATOR_IN_HEAP: u32 = 1u32 << 1;
pub const DECORATOR_NOT_IN_HEAP: u32 = 1u32 << 2;

bitflags! {
    pub struct DecoratorSet : u32 {
        const IN_HEAP = DECORATOR_IN_HEAP;
        const NOT_IN_HEAP = DECORATOR_NOT_IN_HEAP;
    }
}

pub struct AccessAPI<const D: u32>;

impl<const D: u32> AccessAPI<D> {
    pub fn oop_load<Barrier: AccessBarrier, P>(addr: *const P) -> OOP {
        unimplemented!()
    }
    
    pub fn oop_load_at<Barrier: AccessBarrier>(base: OOP, byte_offs: usize) -> OOP {
        unimplemented!()
    }

    pub fn load_at<Barrier: AccessBarrier, T: Copy>(base: OOP,  byte_offs: usize) -> T {
        unimplemented!()
    }

    pub fn oop_store<Barrier: AccessBarrier, P>(addr: *const P, n: OOP) {
        unimplemented!()
    }

    pub fn oop_store_at<Barrier: AccessBarrier>(base: OOP, byte_offs: usize, oop: OOP) {
        unimplemented!()
    }
}

