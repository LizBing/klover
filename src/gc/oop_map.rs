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

use std::{ptr::null_mut, sync::atomic::{AtomicPtr, Ordering}};

pub struct OopMap {
    _offsets: AtomicPtr<Vec<usize>>,
}

impl OopMap {
    pub fn new() -> Self {
        OopMap {
            _offsets: AtomicPtr::new(null_mut()),
        }
    }
}

impl OopMap {
    pub fn resolve(&self, offsets: Vec<usize>) -> bool {
        if self.is_resolved() { return false; }

        let b = Box::new(offsets);
        match self._offsets.compare_exchange(null_mut(), Box::into_raw(b), Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => true,
            Err(_) => false
        }
    }

    pub fn is_resolved(&self) -> bool {
        self._offsets.load(Ordering::SeqCst) != null_mut()
    }
}

impl OopMap {
    pub fn iter(&self) -> std::slice::Iter<'_, usize> {
        assert!(self.is_resolved(), "Should not iterate a unresolved oop map.");

        let v = unsafe { &**self._offsets.as_ptr() };
        v.iter()
    }
}
