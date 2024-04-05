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
use crate::runtime::*;
use std::mem::size_of;

impl Chunk {
    fn new(len: usize) -> AnyObj<Chunk> {
        let c = CHeap_alloc::<Self>(size_of::<Self>() + len);
        c._len = len;
        return c;
    }

    pub fn length(&self) -> usize { return self._len; }

    fn chop(this: AnyObj<Chunk>) {
        this.next_chop();
        dealloc_chunk(this);
    }

    fn next_chop(&self) {
        let mut iter = self.next;
        while let Some(n) = iter {
            iter = n.next;
            dealloc_chunk(n);
        }
    }
}

struct ChunkPool {
    _top: Option<AnyObj<Chunk>>,
    _size: usize
}
unsafe impl Sync for ChunkPool {}

const _num_pools: usize = 4;
static _pools: [ChunkPool; _num_pools] = [];

impl ChunkPool {
    fn clear(&self) {
        while let Some(n) = self._top {
            self._top = n.next;
        }
    }

    fn select(s: usize) -> Option<&'static mut ChunkPool> {
        for n in _pools {
            if n._size == s {
                return Some(&mut n);
            }
        }
        return None;
    }

    fn push(&mut self, n: AnyObj<Chunk>) {
        assert!(n.length() == self._size, "wrong pool");

        let tc = thread_critical::Guard::new();
        
        n.next = self._top;
        self._top = Some(n);
    }

    fn pop(&mut self) -> Option<AnyObj<Chunk>> {
        let tc = thread_critical::Guard::new();

        if let Some(n) = self._top {
            self._top = n.next;
            return Some(n);
        }
        return None;
    }
}

fn alloc_chunk(len: usize) -> AnyObj<Chunk> {
    let n: Option<AnyObj<Chunk>> = None;

    if let Some(p) = ChunkPool::select(len) {
        n = p.pop();
    }

    if let None = n {
        n = Some(Chunk::new(len));
    }

    return n.unwrap();
}

fn dealloc_chunk(n: AnyObj<Chunk>) {
    if let Some(p) = ChunkPool::select(n.length()) { p.push(n); }
}

impl Arena {
    // a chrono task
    pub fn clear() {
        for n in _pools { n.clear(); }
    }

    fn grow() {}

    pub fn alloc<T>() -> AnyObj<T> {
        loop {}
    }
}

