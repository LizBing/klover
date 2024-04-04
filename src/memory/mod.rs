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

pub mod arena;

extern "C" {
    fn CHeap_allocate(s: usize) -> *mut i8;
    fn CHeap_dealloc(p: *mut i8);
}

pub fn CHeap_alloc<T>(s: usize) -> *mut T {
    unsafe { return CHeap_allocate(s).cast(); }
}

pub fn CHeap_sized_alloc<T>() -> *mut T {
    return CHeap_alloc(std::mem::size_of::<T>());
}

pub fn CHeap_free<T>(p: *mut T) {
    unsafe { CHeap_dealloc(p.cast()); }
}

pub struct Obj<T> {
    _ptr: *mut T,
}

impl<T> std::ops::Deref for Obj<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { return &*self._ptr; }
    }
}

impl<T> std::ops::DerefMut for Obj<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { return &mut *self._ptr; }
    }
}

struct Chunk {
    _len: usize,

    // only visible to 'ChunkPool'
    next: *mut Chunk,
}

struct Arena {
    _top: *mut Chunk,

    _begin: *const u8,
    _end: *const u8
}
