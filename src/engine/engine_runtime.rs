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

use crate::code::method::Method;

pub type StackSlot = u32;
pub type DStackSlot = u64;

pub trait StackSlotType: Copy {}

impl StackSlotType for StackSlot {}
impl StackSlotType for DStackSlot {}

#[derive(Debug)]
pub struct Frame<'a> {
    _last_frame: *const Self,
    _mthd: &'a Method<'a>,

    _locals: *const StackSlot,
}

impl<'a> Frame<'a> {
    pub fn last_frame(&self) -> *const Self {
        self._last_frame
    }

    pub fn method(&self) -> &Method<'a> {
        self._mthd
    }

    pub(super) fn local(&self, index: impl Into<usize>) -> *const StackSlot {
        unsafe { self._locals.add(index.into()) }
    }
}
