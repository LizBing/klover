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

use crate::align_up;

pub type Address = usize;

pub struct WordImpl;
pub type Word = *const WordImpl;

pub struct HeapWordImpl;
pub type HeapWord = *const HeapWordImpl;

#[derive(Debug, Clone, Copy)]
pub struct ByteSize(pub usize);
impl From<WordSize> for ByteSize {
    #[inline]
    fn from(value: WordSize) -> Self {
        Self(value.0 << LOG_BYTES_PER_WORD)
    }
}

impl ByteSize {
    #[inline]
    pub fn value(self) -> usize { self.0 }
}

impl ByteSize {
    #[inline]
    pub const fn size_of<T: Sized>() -> Self {
        Self(size_of::<T>())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WordSize(pub usize);
impl From<ByteSize> for WordSize {
    #[inline]
    fn from(value: ByteSize) -> Self {
        Self(align_up!(value.0, size_of::<Word>()) >> LOG_BYTES_PER_WORD)
    }
}

impl WordSize {
    #[inline]
    pub fn value(self) -> usize { self.0 }
}

pub const LOG_BYTES_PER_WORD: i32 = 3;
pub const LOG_BITS_PER_BYTE: i32 = 3;
pub const LOG_BYTES_PER_INT: i32 = 2;
pub const LOG_BITS_PER_INT: i32 = LOG_BITS_PER_BYTE + LOG_BYTES_PER_INT;

pub const K: usize = 1024;
pub const M: usize = K * K;
pub const G: usize = M * K;

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
