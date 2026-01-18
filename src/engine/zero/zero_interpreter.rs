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

use crate::{code::method::Method, engine::{engine_runtime::StackSlot, zero::{zero_instructions::INS_TABLE, zero_runtime::ZeroRegisters}}, utils::global_defs::Address};

pub struct ZeroInterpreter {
    _stack: *const StackSlot,
    _slots: usize,
}

impl ZeroInterpreter {
    pub fn new(byte_size: usize) -> Self {
        unimplemented!()
    }
}

impl Drop for ZeroInterpreter {
    fn drop(&mut self) {
        unimplemented!()
    }
}

impl ZeroInterpreter {
    pub fn process<'a>(&'a self, mthd: &Method) {
        let mut regs = ZeroRegisters::new(self._stack, self._slots);
        let code_data = mthd.code_data().unwrap();

        regs.create_frame(mthd, code_data.max_locals, code_data.max_stack);
        regs.pc = code_data.code.as_ptr();

        loop {
            let opc = unsafe { *regs.pc };
            INS_TABLE[opc as usize](&mut regs);

            if regs.pc.is_null() {
                break;
            }

            regs.pc = unsafe { regs.pc.add(1) };
        }
    }
}
