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

use cafebabe::bytecode::Opcode;

use crate::{code::method::Method, engine::{engine_globals::INTP_STACK_SIZE, engine_runtime::{SLOTS_PER_REF, StackSlot}}, gc::barrier_set::AccessBarrier, memory::allocation::c_heap_alloc, oops::{access::AccessAPI, oop_hierarchy::OOP}, utils::global_defs::{LOG_BYTES_PER_INT, M}};

struct ZeroInterpreter {
    _stack: *const StackSlot,
}

impl ZeroInterpreter {
    pub fn new() -> Self {
        let stack = unsafe { c_heap_alloc(Self::stack_byte_size(), true) } as _;

        Self {
            _stack: stack
        }
    }
}

impl ZeroInterpreter {
    fn stack_byte_size() -> usize {
        INTP_STACK_SIZE.get_value() * M
    }

    fn stack_slot_size() -> usize {
        Self::stack_byte_size() / size_of::<StackSlot>()
    }
}
