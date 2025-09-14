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
    pub fn set_mark(mem: address, mw: MarkWord) {
        addr_cast::<ObjDesc>(mem).expect("null object pointer.")._mw = mw;
    }

    pub fn set_array_len(mem: address, len: i32) {
        let obj = addr_cast::<ObjDesc>(mem).expect("null object pointer.");
        unsafe { *obj._array_len.get_unchecked_mut(0) = len; }
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
        unsafe {
            *self._array_len.get_unchecked(0)
        }
    }

    pub fn data_base(&self) -> address {
        self._data.as_ptr() as _
    }

    pub fn array_data_base(&self) -> address {
        self.data_base() + size_of::<word_t>()
    }
}
