/*
 * Copyright (c) 2025, Lei Zaakjyu. All rights reserved.
 *
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use crate::util::global_defs::{address, word_t};

use super::mark_word::MarkWord;

#[repr(C)]
pub struct ObjDesc {
    _mark_word: MarkWord,
    _data: [word_t; 0]
}

#[repr(C)]
pub struct ArrayObjDesc {
    _super: ObjDesc,
    _data: [word_t; 0]
}

impl ArrayObjDesc {
    pub fn as_obj(&mut self) -> &mut ObjDesc {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }
}

impl ArrayObjDesc {
    pub fn length(&self) -> i32 {
        unsafe { *(&self._data as *const _ as *const _) }
    }

    pub fn set_length(&self, n: i32) {
        unimplemented!()
    }
}

impl ArrayObjDesc {
    pub fn get<T: Sized>(&self, idx: i32) -> Option<T> {
        unimplemented!()
    }

    pub fn put<T: Sized>(&self, idx: i32, n: T) -> bool {
        unimplemented!()
    }
}

