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

use std::cmp::max;

use crate::{code::method::Method, engine::zero::zero_runtime::ZeroFrameData};

pub type StackSlot = u32;
pub type DStackSlot = u64;

pub trait StackSlotType: Copy {}

impl StackSlotType for StackSlot {}
impl StackSlotType for DStackSlot {}

#[derive(Debug)]
pub struct Frame<'a> {
    _last_frame: *const Self,

    _interpreter_frame_data: Option<ZeroFrameData>,

    _mthd: Option<&'a Method<'a>>,
    _locals: *const StackSlot,
    _max_locals: usize,
}

impl<'a> Frame<'a> {
    pub fn init(
        &'a mut self,

        last_frame: *const Self,
        interpreter_frame_data: Option<ZeroFrameData>,
        
        mthd: Option<&'a Method<'a>>,
        locals: *const StackSlot,
        max_locals: u16
    ) {
        *self = Self {
            _last_frame: last_frame,

            _interpreter_frame_data: interpreter_frame_data,

            _mthd: mthd,
            _locals: locals,
            _max_locals: max_locals as _,
        };
    }
}

impl<'a> Frame<'a> {
    pub fn last_frame(&self) -> *const Self {
        self._last_frame
    }

    pub fn interpreter_frame_data(&self) -> Option<&ZeroFrameData> {
        self._interpreter_frame_data.as_ref()
    }

    pub fn method(&self) -> Option<&Method<'a>> {
        self._mthd
    }

    pub fn local(&self, index: impl Into<usize> + Copy) -> *const StackSlot {
        debug_assert!(index.into() < self._max_locals);

        unsafe { self._locals.add(index.into()) }
    }
}
