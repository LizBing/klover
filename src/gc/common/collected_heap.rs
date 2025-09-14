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

use crate::gc::common::barrier_set::AccessBarriers;
use crate::memory::mem_region::MemRegion;
use crate::oops::obj_desc::ObjDesc;
use crate::utils::global_defs::address;

pub trait CollectedHeap: Send + Sync {
    fn mem_alloc(&self, size: usize) -> address;
    
    fn min_obj_size(&self) -> usize {
        ObjDesc::size_of_normal_desc()
    }

    fn mem_region(&self) -> &MemRegion;
}
