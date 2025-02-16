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

use crate::{runtime::os, util::global_defs::{address, LOG_BYTES_PER_ARCH}};

use super::virt_space::VirtSpace;

type NarrowPtr = u32;

// reserve NarrowPtr { 0 } as null ptr,
// a valid narrow address should be greater than one page size.
pub struct CompressedSpace {
    _base: address,
    _vs: VirtSpace
}

impl CompressedSpace {
    pub fn new(
        base: address,
        size: usize,
        init_commit: usize,
        executable: bool,
        pretouch: bool
    ) -> Result<Self, String> {
        let mut res = Self {
            _base: 0,
            _vs: VirtSpace::new(base, size, init_commit, executable, pretouch)?
        };
        res._base = res._vs.mr().begin() - os::get_page_size();

        Ok(res)
    }
}

impl CompressedSpace {
    pub fn vs(&self) -> &VirtSpace {
        &self._vs
    }

    pub fn vs_mut(&mut self) ->&mut VirtSpace {
        &mut self._vs
    }
}

impl CompressedSpace {
    pub fn resolve<T>(&self, nptr: NarrowPtr) -> &mut T {
        let mut ptr = nptr as address;
        ptr <<= LOG_BYTES_PER_ARCH;

        ptr += self._base;

        unsafe { &mut *(ptr as *mut T) }
    }

    pub fn make_narrow<T>(&self, n: &T) -> NarrowPtr {
        (((n as *const _ as address) - self._base) >> LOG_BYTES_PER_ARCH) as _
    }
}
