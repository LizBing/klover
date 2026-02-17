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
    ops::Deref,
    ptr::{
        self,
        NonNull,
        null_mut
    },
    slice::from_raw_parts,
    sync::atomic::{
        AtomicU16,
        Ordering
    }
};

use crate::utils::global_defs::ByteSize;

#[repr(C)]
pub struct Symbol {
    pub next: *mut Symbol,
    ref_cnt: AtomicU16,

    hash: u16,
    len: u16,

    body: [u8; 2]
}

impl Symbol {
    #[inline]
    pub fn compute_hash(bytes: &[u8]) -> u16 {
        let mut h: u16 = 0;

        for &b in bytes {
            h = h.wrapping_mul(31).wrapping_add(b as _);
        }
        assert!(h != 0);

        h
    }

    #[inline]
    pub fn cal_mem_size(bytes: &[u8]) -> ByteSize {
        ByteSize(size_of::<Self>() + bytes.len())
    }
}

impl Symbol {
    pub fn init(&mut self, bytes: &[u8], hash: u16, perm: bool) {
        *self = Self {
            next: null_mut(),
            ref_cnt: AtomicU16::new(0),
            hash: hash,
            len: bytes.len() as _,
            body: [0; 2]
        };

        if perm { self.mark_as_permenant(); }
        
        let dst = self.as_bytes().as_ptr();
        debug_assert!(dst.is_aligned());
        unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), dst as _, self.len()); }
    }
}

impl Symbol {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { from_raw_parts((self as *const Self).add(1) as _, self.len as usize) }
    }
    
    pub fn len(&self) -> usize {
        self.len as _
    }

    pub fn hash(&self) -> u16 {
        self.hash
    }

    pub fn recyclable(&self) -> bool {
        self.ref_cnt.load(Ordering::Acquire) == 0
    }
}

// helpers
impl Symbol {
    fn mark_as_permenant(&self) {
        self.inc_ref_cnt();
    }

    fn inc_ref_cnt(&self) {
        self.ref_cnt.fetch_add(1, Ordering::AcqRel);
    }

    fn dec_ref_cnt(&self) {
        self.ref_cnt.fetch_sub(1, Ordering::AcqRel);
    }
}

#[derive(Debug)]
pub struct SymbolHandle {
    raw: NonNull<Symbol>
}

impl SymbolHandle {
    pub fn new(n: NonNull<Symbol>) -> Self {
        unsafe {
            n.as_ref().inc_ref_cnt();
        }

        Self {
            raw: n
        }
    }
}

impl SymbolHandle {
    pub fn equals(&self, n: &SymbolHandle) -> bool {
        self.raw == n.raw
    }
}

impl Clone for SymbolHandle {
    fn clone(&self) -> Self {
        Self::new(self.raw)
    }
}

impl Deref for SymbolHandle {
    type Target = Symbol;

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.raw.as_ref()
        }
    }
}

impl Drop for SymbolHandle {
    fn drop(&mut self) {
        unsafe {
            self.raw.as_ref().dec_ref_cnt();
        }
    }
}
