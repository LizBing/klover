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

use std::{
    alloc::Layout,
    ptr::{
        NonNull,
        null_mut
    },
    sync::atomic::AtomicUsize,
    u16
};

use bumpalo::Bump;
use parking_lot::Mutex;

use crate::{
    memory::c_malloc::c_malloc,
    oops::symbol::{
        Symbol,
        SymbolHandle
    },
    utils::global_defs::Word
};

const FIXED_BUCKET_LEN: usize = u16::MAX as usize + 1;

#[derive(Debug)]
struct Bucket {
    head: *mut Symbol,
}

impl Bucket {
    fn new() -> Self {
        Self {
            head: null_mut(),
        }
    }
}

impl Bucket {
    fn push(&mut self, n: &mut Symbol) {
        n.next = self.head;
        self.head = n.next;
    }

    fn pop(&mut self) -> Option<&mut Symbol> {
        if self.head.is_null() { None }
        else {
            let res = unsafe { &mut *self.head };
            self.head = res.next;

            Some(res)
        }
    }

    fn find(&self, bytes: &[u8]) -> Option<NonNull<Symbol>> {
        let mut iter = self.head;
        loop {
            if iter.is_null() { return None; }

            unsafe {
                if (*iter).as_bytes() == bytes {
                    return Some(NonNull::new_unchecked(iter));
                }
                iter = (*iter).next;
            }
        }
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    // Arena for permenant symbols(e.g. symbols from bootstrap class loader)
    arena: Bump,

    // Statistics
    symbol_cnt: AtomicUsize,

    buckets: Vec<Mutex<Bucket>>,
}

unsafe impl Send for SymbolTable {}
unsafe impl Sync for SymbolTable {}

impl SymbolTable {
    pub fn new() -> Self {
        let mut res = Self {
            arena: Bump::new(),

            symbol_cnt: AtomicUsize::new(0),

            buckets: Vec::with_capacity(FIXED_BUCKET_LEN)
        };

        res.buckets.resize_with(FIXED_BUCKET_LEN, || Mutex::new(Bucket::new()));

        res
    }
}

impl SymbolTable {
    pub fn intern(&self, bytes: &[u8], perm: bool) -> SymbolHandle {
        let hash = Symbol::compute_hash(bytes);

        let bucket = self.buckets[hash as usize].lock();
        if let Some(x) = bucket.find(bytes) {
            return SymbolHandle::new(x);
        }

        let symbol;
        let size = Symbol::cal_mem_size(bytes);
        if perm {
            let layout = Layout::from_size_align(size.value(), size_of::<Word>()).unwrap();
            symbol = self.arena.alloc_layout(layout).as_ptr() as *mut Symbol;
            unsafe { (*symbol).init(bytes, hash, true); }
        } else {
            symbol = unsafe { c_malloc(size).as_mut().assume_init_mut() };
            unsafe { (*symbol).init(bytes, hash, false); }
        }

        unsafe { SymbolHandle::new(NonNull::new_unchecked(symbol)) }
    }
}
