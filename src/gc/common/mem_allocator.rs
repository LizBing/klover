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

use crate::common::universe;
use crate::oops::klass_handle::KlassHandle;
use crate::oops::mark_word::MarkWord;
use crate::oops::obj_desc::ObjDesc;
use crate::oops::oop;
use crate::oops::oop::ObjPtr;
use crate::utils::global_defs::{addr_cast, address};

pub trait MemAllocator {
    fn size(&self) -> usize;

    fn klass(&self) -> KlassHandle;

    fn initialize(&self, mem: address);

    fn raw_alloc(&self) -> address {
        universe::heap().mem_alloc(self.size())
    }

    fn finish(&self, mem: address) -> ObjPtr {
        ObjDesc::set_mark(mem, MarkWord::prototype(self.klass()));
        oop::as_oop(mem)
    }

    fn allocate(&self) -> ObjPtr {
        let mem = self.raw_alloc();
        self.initialize(mem);
        self.finish(mem)
    }
}
