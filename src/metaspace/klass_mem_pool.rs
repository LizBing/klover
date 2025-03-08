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

use std::{cell::{RefCell, UnsafeCell}, ptr::null_mut, sync::Mutex};

use crate::{align_up, memory::{basic_allocator::BumpAllocator, compressed_space::CompressedSpace, virt_space::VirtSpace}, object::{klass::Klass, mark_word, obj_desc}, util::{global_defs::{self, G, M}, lock_free_stack::LockFreeStack}};

const KP_EXPAND_SIZE: usize = 16 * M;
const KP_SLOT_ALIGNMENT: usize = 128;
const KP_SLOT_SIZE: usize = align_up!(size_of::<Klass>(), KP_SLOT_ALIGNMENT);
const LOG_KP_SLOT_SIZE: usize = global_defs::log2(KP_SLOT_SIZE);
const KP_VM_SPACE_SIZE: usize = 1 << LOG_KP_SLOT_SIZE + mark_word::KLASS_PTR_BITS as usize;

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
    pub fn alloc(&self) -> Box<Klass<'a>> {
        unsafe {
            if let Some(x) = self._reuse.pop() {
                return Box::from_raw(x);
            }

            let res = self.alloc_fast_path();
            if res != null_mut() { return Box::from_raw(res) }; 
        
            Box::from_raw(self.alloc_slow_path())
        }
    }

    fn alloc_fast_path(&self) -> *mut Klass<'a> {
        let bumper = unsafe { &*self._bumper.get() };

        let res = bumper.par_alloc(KP_SLOT_SIZE);
        if res == 0 { return null_mut(); }

        res as _
    }

    fn alloc_slow_path(&self) -> *mut Klass<'a> {
        let _ign = self._mtx.lock().unwrap();

        let bumper = unsafe { &mut *self._bumper.get() };
        let mut res = bumper.par_alloc(KP_SLOT_SIZE);
        if res == 0 {
            if self._space.borrow_mut().vs_mut().expand_by(KP_EXPAND_SIZE, false) {
                res = bumper.expand_and_par_alloc(KP_EXPAND_SIZE, KP_SLOT_SIZE);
            }

            if res == 0 { return null_mut(); }
        }

        res as _
    }

    pub fn free(&self, n: Box<Klass<'a>>) {
        unsafe { self._reuse.push(&mut *Box::into_raw(n)); }
    }
}

