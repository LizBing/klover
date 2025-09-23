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

use crate::{common::universe, oops::{access::DecoratorSet, oop::ObjPtr}, utils::global_defs::{addr_cast, address, naddr, word_t, LOG_BYTES_PER_ARCH}};

pub trait AccessBarriers<const D: u32> {
    #[inline]
    fn flags() -> DecoratorSet {
        DecoratorSet::from_bits_truncate(D)
    }

    fn oop_load_in_heap(addr: address) -> ObjPtr;
    fn oop_load_not_in_heap(addr: address) -> ObjPtr;
    fn load_at<T: Copy>(oop: ObjPtr, offs: usize) -> T;
    fn oop_load_at(oop: ObjPtr, offs: usize) -> ObjPtr;

    fn oop_store_in_heap(addr: address, value: ObjPtr);
    fn oop_store_not_in_heap(addr: address, value: ObjPtr);
    fn store_at<T: Copy>(oop: ObjPtr, offs: usize, value: T);
    fn oop_store_at(oop: ObjPtr, offs: usize, value: ObjPtr);
}

