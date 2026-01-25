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

use std::{fmt::Debug, ptr::null_mut, sync::atomic::{AtomicPtr, Ordering}};

pub unsafe trait NextPtr<T> {
    fn _next_ptr(&self) -> *mut *const T;
}

pub struct LockFreeStack<T: NextPtr<T>> {
    _top: AtomicPtr<T>
}

impl<T: Debug + NextPtr<T>> Debug for LockFreeStack<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            self.iterate(|x| x.fmt(f).unwrap());

            Ok(())
        }
    }
}

impl<T: NextPtr<T>> LockFreeStack<T> {
    pub const fn new() -> Self {
        Self {
            _top: AtomicPtr::new(null_mut())
        }
    }
}

impl<T: NextPtr<T>> LockFreeStack<T> {
    pub fn push(&self, n: &T) {
        let mut exp = self._top.load(Ordering::SeqCst);

        loop {
            unsafe { *(*n)._next_ptr() = exp };

            match self._top.compare_exchange_weak(exp, n as *const _ as _, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => break,
                Err(x) => exp = x
            }
        }
    }

    pub fn pop(&self) -> Option<&T> {
        let mut exp = self._top.load(Ordering::SeqCst);

        loop {
            if exp == null_mut() { return None; }
            let new_top = unsafe { *(*exp)._next_ptr() };

            match self._top.compare_exchange_weak(exp, new_top as _, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => break,
                Err(x) => exp = x
            }
        }

        unsafe {
            Some(&*exp)
        }
    }

    pub unsafe fn iterate<F: FnMut(&T)>(&self, mut cl: F) -> usize {
        let mut counter = 0;

        let mut iter = self._top.load(Ordering::Relaxed) as *const T;
        loop {
            if iter.is_null() { break; }

            cl(&*iter);

            iter = *(*iter)._next_ptr();
            counter += 1;
        }

        counter
    }
}
