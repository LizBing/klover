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

use std::ptr::null;

use crate::{code::method::Method, engine::engine_runtime::{Frame, StackSlot}};

#[derive(Debug)]
pub struct ZeroFrameData {
    _ret_addr: *const u8,
    _last_boundary: *const StackSlot
}

impl ZeroFrameData {
    fn new(ret_addr: *const u8, last_boundary: *const StackSlot) -> Self {
        Self {
            _ret_addr: ret_addr,
            _last_boundary: last_boundary
        }
    }

    fn restore(&self, regs: &mut ZeroRegisters) {
        regs.pc = self._ret_addr;
        regs.boundary = self._last_boundary;
    }
}

pub(super) struct ZeroRegisters {
    pub sp: *const StackSlot,
    pub bp: *const Frame,

    // We use the sb register for bumping allocation(eg locks, etc.).
    pub sb: *const StackSlot,
    // The boundary between sb and sp.
    pub boundary: *const StackSlot,

    pub pc: *const u8,
}

impl ZeroRegisters {
    pub fn new(stack: *const StackSlot, slots: usize) -> Self {
        unsafe {
            let sp = stack.add(slots);

            Self {
                sp: sp,
                bp: null(),
                sb: stack,
                boundary: null(),
                pc: null()
            }
        }
    }
}

impl ZeroRegisters {
    // slot size
    pub fn create_frame(&mut self, mthd: *const Method, max_locals: u16, max_stack: u16) -> bool {
        unsafe {
            let new_bp = self.sp.byte_sub(size_of::<Frame>());
            let new_sp = new_bp.sub(max_locals as _);
            let new_boundary = new_sp.sub(max_stack as _);

            if new_boundary < self.sb {
                return false;
            }

            let data = ZeroFrameData::new(self.pc, self.boundary);
            let new_frame = new_bp as *mut Frame;
            (*new_frame).init(self.bp, Some(data), mthd, new_sp, max_locals);

            self.sp = new_sp;
            self.bp = new_frame;
            self.boundary = new_boundary;
        }

        true
    }

    pub fn unwind(&mut self) -> bool {
        if self.bp.is_null() {
            return false;
        }

        unsafe {
            let new_bp = (*self.bp).last_frame();
            let new_sp = new_bp.add(1) as *const StackSlot;
        
            (*self.bp).interpreter_frame_data().unwrap().restore(self);

            self.bp = new_bp;
            self.sp = new_sp;
        }
    
        true
    }
}
