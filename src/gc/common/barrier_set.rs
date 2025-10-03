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

use std::ptr::{null, null_mut};

use crate::{common::universe, oops::{access::{AccessOps, DecoratorSet}, oop::ObjPtr}, utils::global_defs::{addr_cast, address, naddr, word_t, LOG_BYTES_PER_ARCH}};

pub trait AccessBarriers<const D: u32> {
    fn oop_load_not_in_heap(addr: address) -> ObjPtr;
    fn load_in_heap_at<T: Copy>(oop: ObjPtr, offs: usize) -> T;
    fn oop_load_in_heap_at(oop: ObjPtr, offs: usize) -> ObjPtr;

    fn oop_store_not_in_heap(addr: address, value: ObjPtr);
    fn store_in_heap_at<T: Copy>(oop: ObjPtr, offs: usize, value: T);
    fn oop_store_in_heap_at(oop: ObjPtr, offs: usize, value: ObjPtr);

    fn oop_cas_in_heap_at(oop: ObjPtr, offs: usize, exp: ObjPtr, des: ObjPtr) -> bool;
    fn oop_cas_not_in_heap(addr: address, exp: ObjPtr, des: ObjPtr) -> bool;
    fn oop_xchg_in_heap_at(oop: ObjPtr, offs: usize, new_value: ObjPtr) -> ObjPtr;
    fn oop_xchg_not_in_heap(addr: address, new_value: ObjPtr) -> ObjPtr;

    fn array_copy_in_heap(dst: ObjPtr, dst_offs: usize,
                          src: ObjPtr, src_offs: usize,
                          elem: usize, length: usize);

    fn oop_array_copy_in_heap(dst: ObjPtr, dst_offs: usize,
                              src: ObjPtr, src_offs: usize,
                              length: usize);

    fn clone_in_heap(dst: ObjPtr, src: ObjPtr, size: usize);
}

