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

use core::ffi::c_void as void;
use std::alloc::Layout;

extern "C" {
    fn CHeap_alloc(s: usize) -> *const void;
    fn CHeap_free(p: *const void);
}

struct CHeap;
unsafe impl std::alloc::Allocator for CHeap {
    fn allocate(&self, layout: std::alloc::Layout) 
    -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        unsafe {
            return Ok(
                std::ptr::NonNull::new_unchecked(
                    CHeap_alloc(layout.size()) as _)
                .unwrap_unchecked()
            );
        }
    }

    fn allocate_zeroed(&self, layout: std::alloc::Layout) 
    -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        return self.allocate(layout);
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, 
                         layout: std::alloc::Layout) {
        CHeap_free(ptr.as_ptr() as _);
    }
}

type KBox<T, A = CHeap> = Box<T, A>;

type NativeArena = *const void;
type NativeArenaMark = *const void;
extern "C" {
    fn new_Arena(init_size: usize) -> NativeArena;
    fn delete_Arena(n: NativeArena);

    fn Arena_alloc(this: NativeArena, size: usize) -> *const void;
    fn Arena_try_free(this: NativeArena, p: *const void, size: usize);
    fn Arena_realloc(this: NativeArena, 
                     p: *const void, old_size: usize, new_size: usize)
    -> *const void;
}

struct Arena {
    _handle: NativeArena
}

impl Arena {
    fn new(init_size: usize) -> Self {
        return Arena {
            _handle: unsafe { new_Arena(init_size) }
        };
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        unsafe { delete_Arena(self._handle); }
    }
}

unsafe impl std::alloc::Allocator for Arena {
    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, 
                         layout: std::alloc::Layout) {
        Arena_try_free(self._handle, ptr.as_ptr() as _, layout.size());
    }

    fn allocate(&self, layout: std::alloc::Layout) 
    -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        unsafe {
            return Ok(
                std::ptr::NonNull::new_unchecked(
                    Arena_alloc(self._handle, layout.size()) as _)
                .unwrap_unchecked()
            );
        }
    }

    fn allocate_zeroed(&self, layout: std::alloc::Layout) 
    -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        self.allocate(layout);
    }

    unsafe fn grow(
            &self,
            ptr: std::ptr::NonNull<u8>,
            old_layout: std::alloc::Layout,
            new_layout: std::alloc::Layout,
        ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        return Ok(
            std::ptr::NonNull::new_unchecked(
                Arena_realloc(self._handle, ptr.as_ptr() as _, 
                              old_layout.size(), new_layout.size()) as _
            ).unwrap_unchecked()
        );
    }

    unsafe fn grow_zeroed(
            &self,
            ptr: std::ptr::NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        return self.grow(ptr, old_layout, new_layout);
    }

    unsafe fn shrink(
            &self,
            ptr: std::ptr::NonNull<u8>,
            old_layout: Layout,
            new_layout: Layout,
        ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        return Ok(
            std::ptr::NonNull::new_unchecked(
                Arena_realloc(self._handle, ptr.as_ptr() as _, 
                              old_layout.size(), new_layout.size()) as _
            ).unwrap_unchecked()
        );
    }
}

struct ArenaMark {}

