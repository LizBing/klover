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

pub trait NextPtr<T> {
    fn next_ptr(&mut self) -> *mut *const T;
}

pub struct LockFreeStack<T: NextPtr<T>> {
    _top: AtomicPtr<T>
}

impl<T: NextPtr<T>> LockFreeStack<T> {
    pub fn new() -> Self {
        Self {
            _top: AtomicPtr::new(null_mut())
        }
    }
}

impl<T: NextPtr<T>> LockFreeStack<T> {
    pub fn push(&self, n: &mut T) {
        let mut exp = self._top.load(Ordering::SeqCst);
        loop {
            unsafe { *n.next_ptr() = exp; }
            match self._top.compare_exchange_weak(exp, n as *mut _, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => break,
                Err(x) => exp = x
            }
        }
    }

    pub fn pop(&self) -> Option<&mut T> {
        let mut exp = self._top.load(Ordering::SeqCst);
        loop {
            if exp == null_mut() { return None; }
            let new_top = unsafe { *(*exp).next_ptr() };

            match self._top.compare_exchange_weak(exp, new_top as *mut _, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => break,
                Err(x) => exp = x
            }
        }

        unsafe {
            Some(&mut *exp)
        }
    }
}
