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

use std::ptr::NonNull;

use crate::{
    classfile::class_loader_data::CLDHandle,
    oops::rt_cp_items::RtCPItem,
    utils::{
        global_defs::ByteSize,
        handle::Handle
    }
};

struct RtConstantPool {
    len: usize,
}

impl RtConstantPool {
    fn init<F: FnOnce(&mut Self)>(&mut self, len: usize, f: F) {
        self.len = len;
        f(self);
    }
}

impl RtConstantPool {
    pub fn index(&self, i: usize) -> &RtCPItem {
        assert!(i < self.len);

        unsafe {
            &*((self as *const Self).add(1) as *const RtCPItem).add(i)
        }
    }

    fn index_mut(&self, i: usize) -> &mut RtCPItem {
        unsafe {
            &mut *(self.index(i) as *const _ as *mut _)
        }
    }
}

struct RtCPBuilder {
    len: usize,
    buffer: Vec<RtCPItem>,
}

impl RtCPBuilder {
    pub fn new() -> Self {
        Self {
            len: 0,
            buffer: Vec::new()
        }
    }
}

impl RtCPBuilder {
    pub fn create_rt_cp_item(&mut self, item: RtCPItem) -> usize {
        let res = self.len;
        self.len += 1;

        self.buffer[res] = item;

        res
    }
}

impl RtCPBuilder {
    fn verify_top_value(&self) {
        assert!(self.len < u16::MAX as _);
    }

    pub fn build(mut self, cld: CLDHandle) -> Handle<RtConstantPool> {
        self.verify_top_value();

        let size = ByteSize(size_of::<usize>() + size_of::<RtCPItem>() * self.len);
        let mut mem = unsafe  { NonNull::new_unchecked(cld.mem_alloc_with_size(size).as_ptr() as *mut RtConstantPool) };

        let mut index = self.buffer.len() - 1;
        unsafe {
            mem.as_mut().init(self.buffer.len(), |x| {
                loop {
                    let item = self.buffer.pop();
                    if item.is_none() { break; }

                    *x.index_mut(index) = item.unwrap_unchecked();

                    index -= 1;
                }
            });
        }

        assert!(index == 0 && self.buffer.is_empty(), "Buffer should be empty now.");

        unsafe { Handle::new(mem) }
    }
}
