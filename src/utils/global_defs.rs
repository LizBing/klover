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

use crate::heap_word_align_up;

pub type Address = usize;

pub struct WordImpl;
pub type Word = *const WordImpl;

pub struct HeapWordImpl;
pub type HeapWord = *const HeapWordImpl;

pub const fn word_size_of<T: Sized>() -> usize {
    into_word_size(size_of::<T>())
}

pub const fn into_word_size(byte_size: usize) -> usize {
    heap_word_align_up!(byte_size) >> LOG_BYTES_PER_WORD
}

pub const fn into_byte_size(word_size: usize) -> usize {
    word_size << LOG_BYTES_PER_WORD
}

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

pub trait JavaPrimType: Copy {}
macro_rules! define_java_prim_types {
    ($(($name:ident, $type:ty),)*) => {
        $(
            pub type $name = $type;
            impl JavaPrimType for $name {}
        )*
    };
}

define_java_prim_types! {
    (JByte, i8),
    (JChar, u16),
    (JShort, i16),
    (JInt, i32),
    (JLong, i64),
    (JFloat, f32),
    (JDouble, f64),
}
