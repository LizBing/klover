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
use crate::common::universe;
use crate::metaspace::klass_cell::KlassCell;
use crate::oops::klass::{Klass};
use crate::oops::mark_word;
use crate::oops::mark_word::MarkWord;
use crate::utils::global_defs::{addr_cast, address, word_t};

#[repr(C)]
pub struct ObjDesc {
    _mw: MarkWord,
    _array_len: [i32; 0],
    _data: [u8; 0],
}

impl ObjDesc {
    pub fn init(&mut self, klass: Option<KlassCell>) {
        let klass_ptr = match klass {
            Some(k) => {
                universe::klass_mem_space().compress(k.raw())
            }
            None => 0
        };

        let mut mw_val = 0u64;
        mw_val = MarkWord::set_lock(mw_val, mark_word::UNLOCKED_VALUE);
        mw_val = MarkWord::unset_biased_tag(mw_val);
        mw_val = MarkWord::set_age(mw_val, 0);
        mw_val = MarkWord::set_klass_ptr(mw_val, klass_ptr);
        mw_val = MarkWord::set_hash(mw_val, 0);

        self._mw = MarkWord::with_value(mw_val);
    }
}

impl ObjDesc {
    pub fn size_of_normal_desc() -> usize {
        size_of::<ObjDesc>()
    }

    pub fn size_of_array_desc() -> usize {
        size_of::<ObjDesc>() + size_of::<word_t>()
    }
}

impl ObjDesc {
    pub fn array_len(&self) -> i32 {
        self._array_len[0]
    }

    pub fn data_base(&self) -> address {
        self._data.as_ptr() as _
    }

    pub fn array_data_base(&self) -> address {
        self.data_base() + size_of::<word_t>()
    }
}

impl ObjDesc {
    pub fn array_get<T: Copy>(&self, index: i32) -> Option<T> {
        if index < self.array_len() {
            let addr = self.array_data_base() + size_of::<T>() * (index as usize);
            Some(*addr_cast::<T>(addr).unwrap())
        } else { None }
    }

    pub fn array_set<T: Copy>(&self, index: i32, value: T) -> bool {
        if index < self.array_len() {
            let addr = self.array_data_base() + size_of::<T>() * (index as usize);
            *addr_cast::<T>(addr).unwrap() = value;

            true
        } else { false }
    }
}
