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

use crate::memory::*;
use std::mem::size_of;

impl Chunk {
    pub fn new(len: usize) -> &'static mut Chunk {
        let c = CHeap_alloc::<Chunk>(size_of::<Chunk>() + len);
        unsafe {
            (*c)._len = len;

            return &mut *c;
        }
    }

    pub fn length(&self) -> usize { return self._len; }
}

struct ChunkPool {
    _top: *mut Chunk,
    _size: usize
}

unsafe impl Sync for ChunkPool {}

const _num_pools: usize = 4;
static _pools: [ChunkPool; _num_pools] = [];

impl ChunkPool {
    fn select(s: usize) -> Result<&'static ChunkPool, ()> {
        for i in 0.._num_pools {
            if _pools[i]._size == s {
                return Ok(&_pools[i]);
            }
        }
        return Err(());
    }

}
