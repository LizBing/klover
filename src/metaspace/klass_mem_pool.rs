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

use std::{cell::{RefCell, UnsafeCell}, sync::Mutex};

use crate::{align_up, memory::{basic_allocator::BumpAllocator, compressed_space::CompressedSpace, virt_space::VirtSpace}, object::klass::Klass, util::{global_defs::{G, M}, lock_free_stack::LockFreeStack}};

const KP_EXPAND_SIZE: usize = 16 * M;
const KP_SLOT_ALIGNMENT: usize = 128;
const KP_SLOT_SIZE: usize = align_up!(size_of::<Klass>(), KP_SLOT_ALIGNMENT);
const KP_VM_SPACE_SIZE: usize = 8 * G;

pub struct KlassMemPool<'a> {
    _reuse: LockFreeStack<Klass<'a>>,
    _bumper: UnsafeCell<BumpAllocator>,
    _space: RefCell<CompressedSpace>,

    _mtx: Mutex<()>,
}

impl<'a> KlassMemPool<'a> {
    pub fn new() -> Result<Self, String> {
        let mut res = Self {
            _reuse: LockFreeStack::new(),
            _bumper: UnsafeCell::new(BumpAllocator::new()),
            _space: RefCell::new(CompressedSpace::new(0, KP_VM_SPACE_SIZE, KP_EXPAND_SIZE, false, true)?),
            _mtx: Mutex::new(()),
        };

        let mr = res._space.borrow().vs().mr();
        res._bumper.get_mut().init_with_mr(mr);

        Ok(res)
    }
}

impl<'a> KlassMemPool<'a> {
    pub fn alloc(&self) -> Option<&'a mut Klass> {
        if let Some(x) = self._reuse.pop() {
            return Some(x);
        }

        if let Some(x) = self.alloc_fast_path() {
            return Some(x);
        }

        self.alloc_slow_path()
    }

    fn alloc_fast_path(&self) -> Option<&mut Klass> {
        let bumper = unsafe { &mut *self._bumper.get() };

        let res = bumper.par_alloc(KP_SLOT_SIZE);
        if res == 0 { return None; }

        Some(unsafe { &mut *(res as *mut _) })
    }

    fn alloc_slow_path(&self) -> Option<&mut Klass> {
        let _ign = self._mtx.lock().unwrap();

        let bumper = unsafe { &mut *self._bumper.get() };
        let mut res = bumper.par_alloc(KP_SLOT_SIZE);
        if res == 0 {
            self._space.borrow_mut().vs_mut().expand_by(KP_EXPAND_SIZE, false);
            bumper.expand_by(KP_EXPAND_SIZE);

            res = bumper.par_alloc(KP_SLOT_SIZE);
            if res == 0 { return None; }
        }

        Some(unsafe { &mut *(res as *mut _) })
    }

    pub fn free(&self, n: &mut Klass<'a>) {
        self._reuse.push(n);
    }
}

