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

use std::{slice::from_raw_parts, sync::atomic::AtomicU32};

#[repr(C)]
pub struct Symbol {
    hash: AtomicU32,
    len: u16,
    body: [u8; 2]
}

impl Symbol {
    #[inline]
    fn compute_hash(bytes: &[u8]) -> u32 {
        let mut h: u32 = 0;

        for &b in bytes {
            h = h.wrapping_mul(31).wrapping_add(b as u32);
        }
        assert!(h != 0);

        h
    }
}

impl Symbol {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { from_raw_parts(&self.body[0], self.len as usize) }
    }
    
    pub fn len(&self) -> usize {
        self.len as _
    }

    pub fn hash(&self) -> u32 {
        let h = self.hash.load(std::sync::atomic::Ordering::Relaxed);
        if h != 0 { return h; }

        let computed = Self::compute_hash(self.as_bytes());
        self.hash.store(computed, std::sync::atomic::Ordering::Relaxed);

        computed
    }
}


