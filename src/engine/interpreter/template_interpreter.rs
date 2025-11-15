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

use crate::{code::method::Method, utils::global_defs::Address};

#[derive(Clone)]
pub struct DispatchTable {
    _table: [Address; 1usize << u8::BITS]
}

impl DispatchTable {
    pub fn new() -> Self {
        unimplemented!()
    }
}

type OPStackSlot = isize;

#[derive(Clone)]
pub struct TemplateInterpreter {
    _dispatch_table: DispatchTable,

    _slots: *const OPStackSlot
}


