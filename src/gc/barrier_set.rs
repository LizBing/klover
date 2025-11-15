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

use crate::oops::oop_hierarchy::{ArrayOOP, OOP};

pub trait AccessBarrier<const D: u32> {
    fn load_in_heap_at<T: Copy>(base: OOP, offs_in_bytes: usize) -> T;
    fn store_in_heap_at<T: Copy>(base: OOP, offs_in_bytes: usize, value: T);

    fn cmp_xchg_in_heap_at<T: Copy>(base: OOP, offs_in_bytes: usize, exp: T, des: T)
        -> Result<T, T>;

    fn xchg_in_heap_at<T: Copy>(base: OOP, offs_in_bytes: usize, new: T) -> T;

    fn array_copy_in_heap<T: Copy>(
        src_obj: ArrayOOP, src_offs_in_bytes: usize,
        dst_obj: ArrayOOP, dst_offs_in_bytes: usize,
        length: usize
    );

    fn oop_load_in_heap_at(base: OOP, offs_in_bytes: usize) -> OOP;
    fn oop_store_in_heap_at(base: OOP, offs_in_bytes: usize, oop: OOP);

    fn oop_cmp_xchg_in_heap_at(base: OOP, offs_in_bytes: usize, exp: OOP, des: OOP)
        -> Result<OOP, OOP>;
    fn oop_xchg_in_heap_at(base: OOP, offs_in_bytes: usize, new: OOP) -> OOP;

    fn oop_array_copy_in_heap(
        src_obj: ArrayOOP, src_offs_in_bytes: usize,
        dst_obj: ArrayOOP, dst_offs_in_bytes: usize,
        length: usize
    );

    fn oop_load_not_in_heap<T>(addr: *const T) -> OOP;
    fn oop_store_not_in_heap<T>(addr: *mut T, oop: OOP);

    fn oop_cmp_xchg_not_in_heap<T>(addr: *mut T, exp: OOP, des: OOP)
        -> Result<OOP, OOP>;
    fn oop_xchg_not_in_heap<T>(addr: *mut T, new: OOP) -> OOP;

    fn clone_in_heap(src: OOP, dst: OOP, size_in_bytes: usize);
}
