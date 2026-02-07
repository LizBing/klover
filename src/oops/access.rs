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

use bitflags::bitflags;

use crate::oops::oop_hierarchy::OOP;

macro_rules! define_decorators {
    ($(($x:ident, $shift:expr))*) => {
        paste::paste! {
            $(
                pub const [<DECORATOR_ $x>]: u32 = 1u32 << $shift;
            )*

            bitflags! {
                pub struct DecoratorSet: u32 {
                    $(
                        const $x = [<DECORATOR_ $x>];
                    )*
                }
            }
        }
    };
}

define_decorators! {
    (NONE, 0)

    // JMM flags
    (MO_VOLATILE, 1)    // Ordering::Relaxed
    (MO_ACQUIRE, 2)
    (MO_RELEASE, 3)
    // We use AcqRel for CAS success, Relaxed for CAS failure.
    // For CAS, just pass VOLATILE.

    (INTERNAL_NONCOMPRESSED, 4)

    (IN_HEAP, 5)
    (IN_NATIVE, 6)
}

// offset: byte-unit
pub struct Access<const D: u32>;
impl<const D: u32> Access<D> {
    // addr: *const NarrowOOP
    #[inline]
    pub fn oop_load<T>(addr: *const T) -> OOP {
        unimplemented!()
    }

    #[inline]
    pub fn load_at<T: Copy>(base: OOP, offset: usize) -> T {
        unimplemented!()
    }

    #[inline]
    pub fn oop_load_at(base: OOP, offset: usize) -> OOP {
        unimplemented!()
    }

    // addr: *const NarrowOOP
    #[inline]
    pub fn oop_store<T>(addr: *mut T, oop: OOP) {
        unimplemented!()
    }

    #[inline]
    pub fn store_at<T>(base: OOP, offset: usize, value: T) {
        unimplemented!()
    }

    #[inline]
    pub fn oop_store_at(base: OOP, offset: usize, oop: OOP) {
        unimplemented!()
    }

    // Ordering for CAS: AcqRel for success, Relaxed for failure.

    #[inline]
    pub fn cas_32_at<T>(base: OOP, offset: usize, exp: T, des: T) -> Result<T, T> {
        unimplemented!()
    }
    
    #[inline]
    pub fn cas_32_weak_at<T>(base: OOP, offset: usize, exp: T, des: T) -> Result<T, T> {
        unimplemented!()
    }

    #[inline]
    pub fn cas_64_at<T>(base: OOP, offset: usize, exp: T, des: T) -> Result<T, T> {
        unimplemented!()
    }
    
    #[inline]
    pub fn cas_64_weak_at<T>(base: OOP, offset: usize, exp: T, des: T) -> Result<T, T> {
        unimplemented!()
    }
}
