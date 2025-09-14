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

    fn load_at<T: Copy>(slot: &mut address, offs: usize) -> T;
    fn oop_load(slot: &mut address) -> address;
    fn oop_load_at(slot: &mut address, offs: usize) -> address;

    fn store_at<T: Copy>(slot: &mut address, offs: usize, value: T);
    fn oop_store(slot: &mut address, value: address);
    fn oop_store_at(slot: &mut address, offs: usize, value: address);
}

pub struct NoBarrier;

impl<const D: u32> AccessBarriers<D> for NoBarrier {
    #[inline]
    fn load_at<T: Copy>(slot: &mut address, offs: usize) -> T {
       *addr_cast(*slot + offs).expect("null pointer exception")
    }

    #[inline]
    fn oop_load(slot: &mut address) -> address {
        unimplemented!()
    }

    #[inline]
    fn oop_load_at(slot: &mut address, offs: usize) -> address {
        unimplemented!()
    }
}

