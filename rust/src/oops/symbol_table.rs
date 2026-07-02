use std::{
    hash::{Hash, Hasher},
    ptr::{NonNull, null_mut},
    sync::atomic::{AtomicU32, Ordering},
};

use parking_lot::Mutex;

pub(super) struct Symbol {
    next: *mut Symbol,
    ref_cnt: AtomicU32,

    utf8: String,
}

impl Symbol {
    pub fn utf8(&self) -> &str {
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
    symbols: Mutex<*mut Symbol>,
}

unsafe impl Send for Bucket {}

unsafe impl Sync for Bucket {}

impl Bucket {
    fn intern(&self, n: &str) -> SymbolHandle {
        let s = Symbol {
            next: null_mut(),
            ref_cnt: AtomicU32::new(1),
            utf8: n.into(),
        };

        let mut guard = self.symbols.lock();

        let mut iter = *guard;
        loop {
            if iter != null_mut() {
                unsafe {
                    if (*iter).utf8 == s.utf8 {
                        (*iter).inc_ref_cnt();
                        return SymbolHandle {
                            symbol: NonNull::new_unchecked(iter),
                        };
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
            } else {
                break;
            }
        }

        let pinned = Box::leak(Box::new(s));
        pinned.next = *guard;
        *guard = pinned;

        unsafe {
            SymbolHandle {
                symbol: NonNull::new_unchecked(pinned),
            }
        }
    }
}

const BUCKETS_COUNT: usize = 65536;
static BUCKETS: [Bucket; BUCKETS_COUNT] = [const {
    Bucket {
        symbols: Mutex::new(null_mut()),
    }
}; BUCKETS_COUNT];

pub struct SymbolTable;

fn hash(n: &str) -> usize {
    let mut hash: u32 = 0;
    for &byte in n.as_bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }

    (hash as usize) & (BUCKETS_COUNT - 1)
}

impl SymbolTable {
    pub fn intern(n: &str) -> SymbolHandle {
        BUCKETS[hash(&n)].intern(n)
    }
}

#[derive(Debug)]
pub struct SymbolHandle {
    pub(super) symbol: NonNull<Symbol>,
}

impl Clone for SymbolHandle {
    fn clone(&self) -> Self {
        unsafe {
            self.symbol.as_ref().inc_ref_cnt();
        }
        Self {
            symbol: self.symbol,
        }
    }
}

impl Drop for SymbolHandle {
    fn drop(&mut self) {
        unsafe {
            self.symbol.as_ref().dec_ref_cnt();
        }
    }
}

impl PartialEq for SymbolHandle {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

impl Eq for SymbolHandle {}

impl Hash for SymbolHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.symbol.as_ptr().hash(state);
    }
}

impl From<&str> for SymbolHandle {
    fn from(value: &str) -> Self {
        SymbolTable::intern(value)
    }
}

impl SymbolHandle {
    pub fn equals(&self, n: &Self) -> bool {
        self.symbol == n.symbol
    }

    pub fn utf8(&self) -> &str {
        unsafe { self.symbol.as_ref().utf8() }
    }
}

// ── tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_same_string_returns_same_pointer() {
        let a = SymbolTable::intern("Hello");
        let b = SymbolTable::intern("Hello");
        assert!(a.equals(&b));
    }

    #[test]
    fn intern_different_strings_return_different_pointers() {
        let a = SymbolTable::intern("foo");
        let b = SymbolTable::intern("bar");
        assert!(!a.equals(&b));
    }

    #[test]
    fn intern_same_content_different_cases() {
        let a = SymbolTable::intern("Abc");
        let b = SymbolTable::intern("abc");
        assert!(!a.equals(&b));
        assert_eq!(a.utf8(), "Abc");
        assert_eq!(b.utf8(), "abc");
    }

    #[test]
    fn utf8_returns_correct_string() {
        let h = SymbolTable::intern("HelloWorld");
        assert_eq!(h.utf8(), "HelloWorld");
    }

    #[test]
    fn intern_empty_string() {
        let a = SymbolTable::intern("");
        let b = SymbolTable::intern("");
        assert!(a.equals(&b));
        assert_eq!(a.utf8(), "");
    }

    #[test]
    fn clone_shares_pointer() {
        let a = SymbolTable::intern("shared");
        let b = a.clone();
        assert!(a.equals(&b));
        assert_eq!(a.utf8(), b.utf8());
    }

    #[test]
    fn many_distinct_strings() {
        let handles: Vec<_> = (0..100)
            .map(|i| SymbolTable::intern(format!("string_{}", i).as_str()))
            .collect();

        for (i, h) in handles.iter().enumerate() {
            assert_eq!(h.utf8(), &format!("string_{}", i));
        }

        for (i, h) in handles.iter().enumerate() {
            let h2 = SymbolTable::intern(format!("string_{}", i).as_str());
            assert!(h.equals(&h2));
        }
    }

    #[test]
    fn intern_unicode_string() {
        let a = SymbolTable::intern("你好世界");
        let b = SymbolTable::intern("你好世界");
        assert!(a.equals(&b));
        assert_eq!(a.utf8(), "你好世界");
    }

    #[test]
    fn intern_long_string() {
        let long = "a".repeat(10000);
        let a = SymbolTable::intern(long.as_str());
        let b = SymbolTable::intern(long.as_str());
        assert!(a.equals(&b));
        assert_eq!(a.utf8().len(), 10000);
    }

    #[test]
    fn drop_and_reintern() {
        let s = "drop_and_reintern_test";
        {
            let _a = SymbolTable::intern(s);
        }
        let b = SymbolTable::intern(s);
        assert_eq!(b.utf8(), s);
    }

    #[test]
    fn multiple_clones_then_drop() {
        let a = SymbolTable::intern("multi_clone");
        let b = a.clone();
        let c = a.clone();
        let d = b.clone();

        assert!(a.equals(&c));
        assert!(b.equals(&d));
        assert_eq!(c.utf8(), "multi_clone");

        drop(a);
        drop(b);
        drop(c);
        drop(d);
    }

    #[test]
    fn intern_keeps_original_string() {
        let mut s = String::from("original");
        let h = SymbolTable::intern(s.as_str());
        s.push_str("_modified");
        assert_eq!(h.utf8(), "original");

        let h2 = SymbolTable::intern(s.as_str());
        assert!(!h.equals(&h2));
        assert_eq!(h2.utf8(), "original_modified");
    }
}
