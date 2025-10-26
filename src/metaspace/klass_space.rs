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

use std::ffi::c_void;

unsafe extern "C" {
    unsafe fn KlassSpace_initialize(log_slot_byte_size: usize);

    unsafe fn KlassSpace_allocate() -> *mut c_void;

    unsafe fn KlassSpace_base() -> *mut c_void;
}

struct KlassSpace;
impl KlassSpace {
    // pub fn KlassSpace_encode()
}
