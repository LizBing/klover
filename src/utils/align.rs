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

#[macro_export]
macro_rules! is_aligned {
    ($n: expr, $alignment: expr) => {
        ($n % $alignment == 0) 
    };
}

#[macro_export]
macro_rules! is_arch_aligned {
    ($n: expr) => {
        crate::is_aligned!($n, size_of::<usize>())
    };
}

#[macro_export]
macro_rules! is_page_aligned {
    ($n: expr) => {
        crate::is_aligned!($n, crate::memory::virt_space::VirtSpace::page_size())
    };
}

#[macro_export]
macro_rules! align_up {
    ($n: expr, $a: expr) => {
        (($n + (($a) - 1)) & !(($a) - 1))
    };
}

#[macro_export]
macro_rules! align_down {
    ($n: expr, $a: expr) => {
        ($n & !(($a) - 1))
    };
}

#[macro_export]
macro_rules! heap_word_align_up {
    ($n:expr) => {
        crate::align_up!($n, size_of::<crate::utils::global_defs::HeapWord>())
    };
}
