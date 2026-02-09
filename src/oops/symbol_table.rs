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

use std::{ops::IndexMut, ptr::{NonNull, null, null_mut}};

use crate::{classfile::class_loader_data::ClassLoaderData, oops::symbol::Symbol, utils::global_defs::ByteSize};

const INIT_BUCKET_COUNT: usize = 256;

#[derive(Debug)]
struct Bucket {
    head: *mut Symbol,
    len: usize
}

impl Bucket {
    fn push(&mut self, n: &mut Symbol) {
        n.next = self.head;
        self.head = n;

        self.len += 1
    }

    fn pop(&mut self) -> *mut Symbol {
        let res = self.head;
        if res.is_null() { return null_mut() }

        unsafe {
            self.head = (*res).next;
        }

        res
    }

    fn find(&self, bytes: &[u8]) -> *mut Symbol {
        let mut iter = self.head;
        unsafe {
            loop {
                if iter.is_null() { return null_mut() }
                if (*iter).as_bytes() == bytes { return iter }

                iter = (*iter).next;
            }
        }
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    buckets: Vec<Bucket>,
    mask: usize,

    symbol_count: usize,
    max_bucket_len_index: usize
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut res = Self {
            buckets: Vec::new(),
            mask: INIT_BUCKET_COUNT - 1,
            symbol_count: 0,
            max_bucket_len_index: 0,
        };

        res.buckets.resize_with(INIT_BUCKET_COUNT, || Bucket { head: null_mut(), len: 0 });
    
        res
    }
}

impl SymbolTable {
    pub async fn intern(&mut self, cld: &mut ClassLoaderData, bytes: &[u8]) -> NonNull<Symbol> {
        let hash = Symbol::compute_hash(bytes);
        let bucket_index: usize = (hash % self.mask as u32) as usize;

        let bucket = &mut self.buckets[bucket_index];
        let attempt = bucket.find(bytes);
        if !attempt.is_null() { return unsafe { NonNull::new_unchecked(attempt) } }

        let mem = cld.mem_alloc_with_size(ByteSize(size_of::<Symbol>() + bytes.len())).await;
        let new_symbol = unsafe { &mut *(mem.as_ptr() as *mut Symbol) };
    
        new_symbol.init(bytes, hash);

        bucket.push(new_symbol);
        self.symbol_count += 1;
        if bucket.len > self.buckets[self.max_bucket_len_index].len {
            self.max_bucket_len_index = bucket_index;
        }

        unsafe { NonNull::new_unchecked(new_symbol) }
    }
}

impl SymbolTable {
    fn need_rehash(&self) -> bool {
        self.symbol_count > self.buckets.len() || self.buckets[self.max_bucket_len_index].len > 8
    }

    pub fn try_rehash(&mut self) -> bool {
        if !self.need_rehash() { return false }

        let old_buckets = &mut self.buckets;

        let new_len = old_buckets.len() * 2;
        self.mask = new_len - 1;

        let mut new_buckets = Vec::with_capacity(new_len);
        new_buckets.resize_with(new_len, || Bucket { head: null_mut(), len: 0 });

        self.max_bucket_len_index = 0;
        for old_bucket in old_buckets {
            loop {
                let n = old_bucket.pop();
                if n.is_null() { break; }
                let symbol = unsafe { &mut *n };

                let new_bucket_index = (symbol.hash() % self.mask as u32) as usize;
                let new_bucket = &mut new_buckets[new_bucket_index];

                new_bucket.push(symbol);
                if new_bucket.len > new_buckets[self.max_bucket_len_index].len {
                    self.max_bucket_len_index = new_bucket_index;
                }
            }
        }

        self.buckets = new_buckets;

        true
    }
}
