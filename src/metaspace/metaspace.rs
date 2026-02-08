/*
 * Copyright 2026 Lei Zaakjyu
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

use crate::{init_ll, memory::{compressed_space::CompressedSpace, mem_region::MemRegion, virt_space::VirtSpace}, runtime::universe::Universe, utils::{global_defs::{ByteSize, WordSize}, linked_list::{LinkedList, LinkedListNode}}};

#[derive(Debug)]
pub struct Metaspace {
    comp_space: CompressedSpace,
    cs_chunk_list: LinkedList<MetaChunk>,

    // code_space maybe...
}

impl Metaspace {
    pub fn new() -> Self {
        let cvs = VirtSpace::new(WordSize::from(ByteSize(*Universe::vm_flags().xmx)), false);

        Self {
            comp_space: CompressedSpace::new(cvs),
            cs_chunk_list: LinkedList::new()
        }
    }

    pub fn init(&mut self) {
        init_ll!(&mut self.cs_chunk_list, MetaChunk, node)
    }
}

impl Metaspace {}

#[derive(Debug)]
pub struct MetaChunk {
    pub(super) node: LinkedListNode<Self>,
    mr: MemRegion
}
