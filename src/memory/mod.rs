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

extern "C" {
    fn CHeap_alloc(s: usize) -> *mut core::ffi::c_void;
    fn CHeap_free(p: *mut core::ffi::c_void);
}

struct CHeap;
unsafe impl std::alloc::Allocator for CHeap {
    fn allocate(&self, layout: std::alloc::Layout) 
    -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        unsafe {
            return Ok(
                std::ptr::NonNull::new(CHeap_alloc(layout.size()) as *mut _)
                .unwrap_unchecked()
            );
        }
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, 
                         layout: std::alloc::Layout) {
        CHeap_free(ptr.as_ptr() as *mut _);
    }
}

type KBox<T, A = CHeap> = Box<T, A>;

type NativeArena = *const core::ffi::c_void;
type ArenaMark = *const core::ffi::c_void;
extern "C" {
    fn new_Arena(init_size: usize) -> NativeArena;
    fn delete_Arena(n: NativeArena);

    fn Arena_alloc(this: NativeArena, size: usize)
    -> *const core::ffi::c_void;
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
                         layout: std::alloc::Layout) {}

    fn allocate(&self, layout: std::alloc::Layout) 
    -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        ;
    }
}


