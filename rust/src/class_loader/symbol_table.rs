use std::{hash::Hash, iter::from_fn, ptr::{NonNull, null_mut}, sync::atomic::{AtomicU32, Ordering}};

use parking_lot::{Mutex, lock_api::Mutex};

use crate::class_loader::symbol_handle::SymbolHandle;

pub struct Symbol {
    next: *mut Symbol,
    ref_cnt: AtomicU32,
    
    utf8: String
}

impl Symbol {
    pub fn utf8(&self) -> &String {
        &self.utf8
    }
}

impl Symbol {
    pub fn inc_ref_cnt(&self) {
        self.ref_cnt.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_ref_cnt(&self) {
        self.ref_cnt.fetch_sub(1, Ordering::Release);
    }

    fn is_recyclable(&self) -> bool {
        self.ref_cnt.load(Ordering::Acquire) == 0
    }
}

#[derive(Debug)]
struct Bucket {
    symbols: Mutex<*mut Symbol>
}

unsafe impl Send for Bucket {}

unsafe impl Sync for Bucket {}

impl Bucket {
    fn intern(&self, n: String) -> SymbolHandle {
        let s = Symbol {
            next: null_mut(),
            ref_cnt: AtomicU32::new(1),
            utf8: n
        };
        
        let mut guard = self.symbols.lock();

        let mut iter = *guard;
        loop {
            if iter != null_mut() {
                unsafe {
                    if (*iter).utf8 == s.utf8 {
                        (*iter).inc_ref_cnt();
                        return SymbolHandle { symbol: NonNull::new_unchecked(iter) }
                    }

                    if (*iter).is_recyclable() {
                        let n = iter;
                        iter = (*iter).next;
                        
                        // Drop here.
                        let _ = Box::from_raw(n);
                    } else {
                        iter = (*iter).next;
                    }
                }
            } else { break; }
        }

        let pinned = Box::leak(Box::new(s));
        pinned.next = *guard;
        *guard = pinned;
        
        unsafe {
            SymbolHandle { symbol: NonNull::new_unchecked(pinned) }
        }
    }
}

const BUCKETS_COUNT: usize = 65536;
static BUCKETS: [Bucket; BUCKETS_COUNT] = [const {
    Bucket { symbols: Mutex::new(null_mut()) }
}; BUCKETS_COUNT];

pub struct SymbolTable;

fn hash(n: &String) -> usize {
    let mut hash: u32 = 0;
    for &byte in n.as_bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }

    (hash as usize) & (BUCKETS_COUNT - 1)
}

impl SymbolTable {
    pub fn intern(n: String) -> SymbolHandle {
        BUCKETS[hash(&n)].intern(n)
    }
}
