/*
 * Copyright (c) 2024, Lei Zaakjyu. All rights reserved.
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

use crate::gc::interface::*;

// the base trait of all managed types
pub trait Object {
    fn size() -> usize;
    fn iter() -> dyn Iterator<Item = crate::off_t>;
}

// ordinary object pointer
struct oop<T: Object> {
    _ptr: address
}

impl<T> oop<T> {
    pub fn addr(&self) -> address {
        return self._ptr;
    }

    ;
}

struct CompressedOop<T: Object> {
    _ptr: compressed_addr
}

impl<T> CompressedOop<T> {
    pub fn addr(&self) -> compressed_addr {
        return self._ptr;
    }

    pub fn set_addr(&mut self) {}
}

