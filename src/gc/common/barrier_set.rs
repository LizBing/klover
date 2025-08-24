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

use crate::{common::universe, oops::oop::ObjPtr, utils::global_defs::{addr_cast, address, naddr, word_t, LOG_BYTES_PER_ARCH}};

bitflags::bitflags! {
    pub struct Decorator: u32 {
        const IN_HEAP       = 1 << 0;
        const COMPRESSED    = 1 << 1;
        const LOAD_BARRIER  = 1 << 2;
        const STORE_BARRIER = 1 << 3;
    }
}

#[inline]
fn encode_coop(addr: address) -> naddr {
    if addr == 0 { return 0; }

    let base = universe::coops_base();
    ((addr - base - size_of::<word_t>()) >> LOG_BYTES_PER_ARCH) as _
}

#[inline]
fn decode_coop(addr: naddr) -> address {
    if addr == 0 { return 0;}
    
    let base = universe::coops_base();
    ((addr as address) << LOG_BYTES_PER_ARCH) + base + size_of::<word_t>()
}
