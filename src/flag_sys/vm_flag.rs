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

use std::cell::{Cell, UnsafeCell};

pub type ConstrainFunc<T> = fn(T) -> bool;

pub struct VMFlagData<T: Copy> {
    _name: &'static str,

    _value: Cell<T>,
    _cons_func: Option<ConstrainFunc<T>>,

    _desc: &'static str,
}

unsafe impl<T: Copy> Sync for VMFlagData<T> {}

impl<T: Copy> VMFlagData<T> {
    pub const fn new(
        name: &'static str, 
        value: T,
        cons_func: Option<ConstrainFunc<T>>,
        desc: &'static str,
    ) -> Self {
        Self {
            _name: name,
            _value: Cell::new(value),
            _desc: desc,
            _cons_func: cons_func
        }
    }
}

impl<T: Copy> VMFlagData<T> {
    pub fn name(&self) -> &str {
        self._name
    }

    pub fn get_value(&self) -> T {
        self._value.get()
    }

    pub unsafe fn set_value(&self, n: T) {
        self._value.set(n);
    }

    pub fn desc(&self) -> &'static str {
        self._desc
    }
}

pub enum VMFlag {
    UsizeFlag(&'static VMFlagData<usize>),
    I32Flag(&'static VMFlagData<i32>),
    BoolFlag(&'static VMFlagData<bool>)
}

