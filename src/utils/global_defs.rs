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

use core::fmt;
use std::ptr::null;

pub type Address = usize;

pub struct HeapWordImpl;
pub type HeapWord = *const HeapWordImpl;

/*
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct HeapAddress(*const HeapWord);

impl HeapAddress {
    pub fn new<T: Into<*const HeapWord>>(n: T) -> Self {
        Self(n.into())
    }

    pub fn as_ptr<T>(self) -> *const T {
        self.0 as _
    }

    pub fn is_null(self) -> bool {
        self.0 == null()
    }

    pub fn equals(left: Self, right: Self) -> bool {
        left.0 == right.0
    }

    pub fn diff_in_bytes(left: Self, right: Self) -> isize {
        left.0 as isize - right.0 as isize
    }

    pub fn diff_in_words(left: Self, right: Self) -> isize {
        Self::diff_in_bytes(left, right) >> LOG_BYTES_PER_WORD
    }

    pub fn delta_in_bytes(left: Self, right: Self) -> usize {
        assert!(left.0 >= right.0, "left({}) should be greater than right({})", left, right);

        left.0 as usize - right.0 as usize
    }

    pub fn delta_in_words(left: Self, right: Self) -> usize {
        Self::delta_in_bytes(left, right) >> LOG_BYTES_PER_WORD
    }

    pub fn offset_in_bytes(self, offs: isize) -> Self {
        Self::new(self.0.wrapping_byte_offset(offs))
    }

    pub fn offset_in_words(self, offs: isize) -> Self {
        Self::new(self.0.wrapping_offset(offs))
    }
}

impl fmt::Display for HeapAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:p}", self.0)
    }
}
*/

pub const LOG_BYTES_PER_WORD: i32 = 3;
pub const LOG_BITS_PER_BYTE: i32 = 3;
pub const LOG_BYTES_PER_INT: i32 = 2;
pub const LOG_BITS_PER_INT: i32 = LOG_BITS_PER_BYTE + LOG_BYTES_PER_INT;

pub const K: usize = 1024;
pub const M: usize = K * K;
pub const G: usize = M * K;

#[macro_export]
macro_rules! PTR_FORMAT {
    () => {
        "{:p}"
    };
}

pub type JByte = i8;
pub type JChar = u16;
pub type JShort = i16;
pub type JInt = i32;
pub type JLong = i64;
pub type JFloat = f32;
pub type JDouble = f64;
