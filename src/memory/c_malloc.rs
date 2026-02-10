/*
 * Copyright 2026 Lei Zaakjyu
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

use std::ptr::NonNull;

use crate::utils::global_defs::ByteSize;

pub unsafe fn c_malloc<T>(size: ByteSize) -> NonNull<T> {
    NonNull::new(libc::malloc(size.value()) as _).expect("out of memory(c heap)")
}

pub unsafe fn c_free<T>(addr: *const T) {
    libc::free(addr as _)
}
