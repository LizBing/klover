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

use std::{ptr::NonNull, sync::atomic::Ordering};

use crate::{align_up, classfile::class_loader_data::ClassLoaderData, init_ll, is_aligned, memory::{bumper::Bumper, compressed_space::{CompressedSpace, NarrowEncoder}, mem_region::MemRegion, virt_space::VirtSpace}, runtime::universe::Universe, utils::{global_defs::{ByteSize, HeapWord, K, WordSize}, linked_list::{LinkedList, LinkedListNode}}};

pub const SMALL_MSCHUNK_SIZE: ByteSize = ByteSize(8 * K);

#[derive(Debug)]
pub struct Metaspace {
    comp_space: CompressedSpace,
    chunk_list: LinkedList<MSChunk>,
    free_list: LinkedList<MSChunk>,
}

impl Metaspace {
    pub fn new(size: ByteSize) -> Self {
        // ensure
        assert!(is_aligned!(SMALL_MSCHUNK_SIZE.value(), VirtSpace::page_size().value()));

        let cvs = VirtSpace::new(size, false);

        Self {
            comp_space: CompressedSpace::new(cvs),
            chunk_list: LinkedList::new(),
            free_list: LinkedList::new()
        }
    }

    pub fn init(&mut self) {
        init_ll!(&mut self.chunk_list, MSChunk, chunk_list_node);
        init_ll!(&mut self.free_list, MSChunk, owning_node);
    }
}

impl Metaspace {
    pub fn create_narrow_encoder(&self) -> NarrowEncoder {
        NarrowEncoder::new(self.comp_space.base())
    }

    pub fn alloc_small_chunk(&mut self) -> NonNull<MSChunk> {
        if let Some(x) = self.free_list.pop_back() {
            x.bumper.clear();
            unsafe {
                return NonNull::new_unchecked(x as *const _ as _);
            }
        }

        let vs = &mut self.comp_space.vs;

        let start = vs.committed().end();
        assert!(vs.expand_by(SMALL_MSCHUNK_SIZE), "out of memory(metaspace)");

        let chunk = Box::leak(Box::new(MSChunk::new(MemRegion::with_size(start, SMALL_MSCHUNK_SIZE.into()))));
        self.chunk_list.push_back(chunk);

        unsafe { NonNull::new_unchecked(chunk) }
    }

    pub fn alloc_sized_chunk(&mut self, size: ByteSize) -> NonNull<MSChunk> {
        let chunk_size = ByteSize(align_up!(size.value(), SMALL_MSCHUNK_SIZE.value()));

        if chunk_size.value() == SMALL_MSCHUNK_SIZE.value() {
            if let Some(x) = self.free_list.pop_back() {
                x.bumper.clear();
                unsafe {
                    return NonNull::new_unchecked(x as *const _ as _);
                }
            }
        }

        let vs = &mut self.comp_space.vs;
        
        let start = vs.committed().end();
        assert!(vs.expand_by(chunk_size), "out of memory(metaspace)");

        let chunk = Box::leak(Box::new(MSChunk::new(MemRegion::with_size(start, chunk_size.into()))));
        self.chunk_list.push_back(chunk);

        unsafe { NonNull::new_unchecked(chunk) }
    }

    pub fn free_chunk(&mut self, mut c: NonNull<MSChunk>) {
        unsafe {
            c.as_mut().owning_node.erase();
            self.free_list.push_back(c.as_mut());
        }
    }
}

impl Metaspace {
    pub fn try_and_alloc_small_chunk(&mut self, cld: &mut ClassLoaderData, size: ByteSize) -> NonNull<HeapWord> {
        unsafe {
            let attempt = cld.try_mem_alloc(size, Ordering::Relaxed);
            if !attempt.is_null() {
                return NonNull::new_unchecked(attempt);
            }

            let new_chunk = self.alloc_small_chunk().as_mut();
            let res = new_chunk.bumper.alloc_with_size(size.into());
            debug_assert!(!res.is_null());

            cld.release_new_chunk(new_chunk);

            NonNull::new_unchecked(res)
        }
    }
}

#[derive(Debug)]
pub struct MSChunk {
    pub(super) chunk_list_node: LinkedListNode<Self>,
    pub owning_node: LinkedListNode<Self>,

    pub bumper: Bumper
}

impl MSChunk {
    fn new(mr: MemRegion) -> Self {
        Self {
            chunk_list_node: LinkedListNode::new(),
            owning_node: LinkedListNode::new(),
            bumper: Bumper::new(mr)
        }
    }
}
