use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ptr;

use parking_lot::{Mutex, RwLock};

const SMALL_CHUNK_BYTE_SIZE: usize = 8 * 1024; // 8 KB
const BUMP_THRESHOLD: usize = SMALL_CHUNK_BYTE_SIZE / 2; // 4 KB

unsafe extern "C" {
    pub fn ms_init() -> bool;
    fn ms_alloc_small_chunk() -> *mut MSChunk;
    fn ms_alloc_sized_chunk(byte_size: usize) -> *mut MSChunk;
    fn ms_free_chunk(chunk: *mut MSChunk);
}

#[repr(C)]
struct MSChunk {
    _next: *mut MSChunk,
    byte_size: usize,
    start: usize,
}

// SAFETY: chunks are allocated by the C metaspace layer and protected
// by internal synchronisation (Mutex / RwLock) in MSAllocator.
unsafe impl Send for MSChunk {}
unsafe impl Sync for MSChunk {}

pub struct MSAllocator {
    chunks: Mutex<Vec<NonNull<MSChunk>>>,
    /// Current chunk used for bump allocation. Protected by RwLock:
    /// read-lock for the fast path (peek + CAS on offset),
    /// write-lock for the slow path (chunk swap).
    cur_chunk: RwLock<Option<NonNull<MSChunk>>>,
    /// Bump-pointer offset inside the current chunk.
    /// Lock-free – allocated via CAS in the fast path.
    cur_offset: AtomicUsize,
}

// SAFETY: all mutable state is behind internal synchronisation
// (Mutex, RwLock, AtomicUsize).
unsafe impl Send for MSAllocator {}
unsafe impl Sync for MSAllocator {}

impl MSAllocator {
    pub const fn new() -> Self {
        MSAllocator {
            chunks: Mutex::new(Vec::new()),
            cur_chunk: RwLock::new(None),
            cur_offset: AtomicUsize::new(0),
        }
    }

    /// Allocate memory for a value of type `T`.
    ///
    /// Objects smaller than `BUMP_THRESHOLD` (4 KB) are allocated via
    /// bump-pointer allocation inside a small chunk.  Larger objects
    /// receive their own dedicated chunk obtained from the C metaspace
    /// layer.
    pub fn alloc<T>(&self, size: usize) -> *mut T {
        if size < BUMP_THRESHOLD {
            self.bump_alloc(size)
        } else {
            self.sized_alloc(size)
        }
    }

    pub fn calloc<T>(&self, elem_size: usize, count: usize) -> *mut T {
        self.alloc(elem_size * count)
    }

    // ── bump-pointer allocation ──────────────────────────────────────
    //
    //  Two-tier strategy:
    //   1. Fast path  – read-lock on cur_chunk + CAS on cur_offset.
    //      No writer blocking; multiple threads can bump in parallel.
    //   2. Slow path  – write-lock on cur_chunk; retry or acquire a
    //      fresh small chunk.

    fn bump_alloc<T>(&self, size: usize) -> *mut T {
        let align = std::mem::align_of::<T>();

        loop {
            // ── fast path ────────────────────────────────────────────
            {
                let cur = self.cur_chunk.read();
                if let Some(chunk) = *cur {
                    let chunk = unsafe { chunk.as_ref() };

                    let old_offset = self.cur_offset.load(Ordering::Relaxed);
                    let aligned = (old_offset + align - 1) & !(align - 1);

                    if aligned + size <= chunk.byte_size {
                        // Try to atomically reserve the space.
                        if self
                            .cur_offset
                            .compare_exchange_weak(
                                old_offset,
                                aligned + size,
                                Ordering::AcqRel,
                                Ordering::Relaxed,
                            )
                            .is_ok()
                        {
                            return (chunk.start + aligned) as *mut T;
                        }
                        // CAS raced with another thread – retry.
                        continue;
                    }
                }
                // No current chunk or it is full → drop read-lock and
                // fall through to the slow path.
            }

            // ── slow path ────────────────────────────────────────────
            return self.bump_alloc_slow::<T>(size);
        }
    }

    /// Slow path: acquire write-lock, check again, and if necessary
    /// obtain a new small chunk from the C metaspace layer.
    fn bump_alloc_slow<T>(&self, size: usize) -> *mut T {
        let align = std::mem::align_of::<T>();
        let mut cur = self.cur_chunk.write();

        // Re-check: another thread may have refreshed the chunk while
        // we were waiting for the write-lock.  Use CAS even here
        // because fast-path threads that read the old chunk pointer
        // before we took the lock may still be racing on cur_offset.
        if let Some(chunk) = *cur {
            let chunk = unsafe { chunk.as_ref() };
            loop {
                let offset = self.cur_offset.load(Ordering::Relaxed);
                let aligned = (offset + align - 1) & !(align - 1);

                if aligned + size <= chunk.byte_size {
                    if self
                        .cur_offset
                        .compare_exchange_weak(
                            offset,
                            aligned + size,
                            Ordering::AcqRel,
                            Ordering::Relaxed,
                        )
                        .is_ok()
                    {
                        return (chunk.start + aligned) as *mut T;
                    }
                    // CAS failed – retry.
                    continue;
                }
                break; // chunk still full
            }
        }

        // Acquire a fresh small chunk.
        let new_chunk = unsafe { NonNull::new(ms_alloc_small_chunk()) }
            .expect("ms_alloc_small_chunk: out of metaspace memory");

        // Record the chunk.
        {
            let mut chunks = self.chunks.lock();
            chunks.push(new_chunk);
        }

        // Install the new chunk and reset offset.
        let chunk = unsafe { new_chunk.as_ref() };
        *cur = Some(new_chunk);
        self.cur_offset.store(size, Ordering::Release);

        chunk.start as *mut T
    }

    // ── sized (large-object) allocation ───────────────────────────────

    fn sized_alloc<T>(&self, size: usize) -> *mut T {
        let chunk = unsafe { NonNull::new(ms_alloc_sized_chunk(size)) }
            .expect("ms_alloc_sized_chunk: out of metaspace memory");

        // Record the chunk.
        {
            let mut chunks = self.chunks.lock();
            chunks.push(chunk);
        }

        unsafe { chunk.as_ref() }.start as *mut T
    }
}

impl Drop for MSAllocator {
    fn drop(&mut self) {
        // All chunks — including the current bump chunk — are tracked
        // in `chunks`.  Drain and free each one through the C layer.
        let chunks = self.chunks.get_mut();
        for chunk in chunks.drain(..) {
            unsafe { ms_free_chunk(chunk.as_ptr()) };
        }
    }
}

impl Default for MSAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// ── MSBox ─────────────────────────────────────────────────────────────

/// A pointer type that owns a heap allocation inside an
/// [`MSAllocator`]'s metaspace arena.
///
/// Individual deallocations are not supported (bump-allocator
/// semantics); memory is reclaimed when the underlying chunks are
/// destroyed together with the allocator.
pub struct MSBox<T> {
    raw: NonNull<T>,
}

impl<T> MSBox<T> {
    /// Allocate memory through `allocator` and move `value` into it.
    pub fn new(allocator: &MSAllocator, value: T) -> Self {
        let size = std::mem::size_of::<T>();
        let ptr = allocator.alloc::<T>(size);
        unsafe { ptr.write(value) };
        MSBox {
            raw: unsafe { NonNull::new_unchecked(ptr) },
        }
    }

    pub fn from_raw(raw: *mut T) -> Self {
        Self {
            raw: NonNull::new(raw).unwrap()
        }
    }
}

impl<T> std::ops::Deref for MSBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.raw.as_ref() }
    }
}

impl<T> std::ops::DerefMut for MSBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.raw.as_mut() }
    }
}

impl<T> Drop for MSBox<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.raw.as_ptr());
        }
    }
}

// SAFETY: MSBox owns a uniquely-allocated region of metaspace memory,
// so it is Send/Sync under the same conditions as Box<T>.
unsafe impl<T: Send> Send for MSBox<T> {}
unsafe impl<T: Sync> Sync for MSBox<T> {}

// Safety: guaranteed by developer.
pub struct MSRef<T> {
    raw: NonNull<T>
}

impl<T> From<&MSBox<T>> for MSRef<T> {
    fn from(value: &MSBox<T>) -> Self {
        Self { raw: value.raw }
    }
}

impl<T> From<&T> for MSRef<T> {
    fn from(value: &T) -> Self {
        unsafe { Self { raw: NonNull::new_unchecked(value as *const T as *mut T) } }
    }
}

impl<T> From<*const T> for MSRef<T> {
    fn from(value: *const T) -> Self {
        unsafe { (&*value).into() }
    }
}

impl<T> Deref for MSRef<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

// ── tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    fn ms_init_once() {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            let ok = unsafe { ms_init() };
            assert!(ok, "ms_init: metaspace initialisation failed");
        });
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C, align(64))]
    struct OverAligned {
        data: [u8; 64],
    }

    #[test]
    fn alloc_basic() {
        ms_init_once();
        let allocator = MSAllocator::new();
        let b = MSBox::new(&allocator, Point { x: 10, y: 20 });
        assert_eq!(b.x, 10);
        assert_eq!(b.y, 20);
    }

    #[test]
    fn alloc_int() {
        ms_init_once();
        let allocator = MSAllocator::new();
        let b = MSBox::new(&allocator, 42u32);
        assert_eq!(*b, 42);
    }

    #[test]
    fn alloc_empty_tuple() {
        ms_init_once();
        let allocator = MSAllocator::new();
        let b = MSBox::new(&allocator, ());
        let _ = *b;
    }

    #[test]
    fn alloc_multiple_distinct_addresses() {
        ms_init_once();
        let allocator = MSAllocator::new();
        let a = MSBox::new(&allocator, 1u64);
        let b = MSBox::new(&allocator, 2u64);
        let c = MSBox::new(&allocator, 3u64);

        let pa = &*a as *const u64;
        let pb = &*b as *const u64;
        let pc = &*c as *const u64;

        assert_ne!(pa, pb);
        assert_ne!(pb, pc);
        assert_ne!(pa, pc);
        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        assert_eq!(*c, 3);
    }

    #[test]
    fn alloc_many_small() {
        ms_init_once();
        let allocator = MSAllocator::new();
        let boxes: Vec<MSBox<u32>> = (0..1000).map(|i| MSBox::new(&allocator, i)).collect();
        for (i, b) in boxes.iter().enumerate() {
            assert_eq!(**b, i as u32);
        }
    }

    #[test]
    fn deref_mut_field() {
        ms_init_once();
        let allocator = MSAllocator::new();
        let mut b = MSBox::new(&allocator, Point { x: 0, y: 0 });
        b.x = 100;
        b.y = 200;
        assert_eq!(b.x, 100);
        assert_eq!(b.y, 200);
    }

    #[test]
    fn alloc_large_object() {
        ms_init_once();
        let allocator = MSAllocator::new();
        let mut b = MSBox::new(&allocator, [0u8; 5 * 1024]);
        for (i, byte) in b.iter_mut().enumerate() {
            *byte = (i & 0xff) as u8;
        }
        for (i, &byte) in b.iter().enumerate() {
            assert_eq!(byte, (i & 0xff) as u8);
        }
    }

    #[test]
    fn alloc_mixed_small_and_large() {
        ms_init_once();
        let allocator = MSAllocator::new();
        let small = MSBox::new(&allocator, 7u64);
        let large = MSBox::new(&allocator, [0xffu8; 5000]);
        let another = MSBox::new(&allocator, 42i32);

        assert_eq!(*small, 7);
        assert_eq!(large[0], 0xff);
        assert_eq!(large[4999], 0xff);
        assert_eq!(*another, 42);
    }

    #[test]
    fn chunk_overflow_forces_new_chunk() {
        ms_init_once();
        let allocator = MSAllocator::new();
        let boxes: Vec<MSBox<[u8; 256]>> = (0..64)
            .map(|i| {
                let mut arr = [0u8; 256];
                arr[0] = i as u8;
                MSBox::new(&allocator, arr)
            })
            .collect();
        for (i, b) in boxes.iter().enumerate() {
            assert_eq!(b[0], i as u8);
        }
    }

    #[test]
    fn alloc_overaligned() {
        ms_init_once();
        let allocator = MSAllocator::new();
        let b = MSBox::new(&allocator, OverAligned { data: [0xAA; 64] });
        let addr = &*b as *const OverAligned as usize;
        assert_eq!(addr % 64, 0);
    }

    #[test]
    fn drop_allocator_does_not_crash() {
        ms_init_once();
        {
            let allocator = MSAllocator::new();
            let _a = MSBox::new(&allocator, 1u32);
            let _b = MSBox::new(&allocator, [0u8; 6000]);
            let _c = MSBox::new(&allocator, 3.14f64);
        }
    }

    #[test]
    fn drop_empty_allocator_does_not_crash() {
        ms_init_once();
        {
            let _allocator = MSAllocator::new();
        }
    }

    #[test]
    fn concurrent_allocations() {
        ms_init_once();
        use std::sync::Arc;

        let allocator = Arc::new(MSAllocator::new());
        let mut handles = Vec::new();

        for tid in 0..8 {
            let a = Arc::clone(&allocator);
            handles.push(std::thread::spawn(move || {
                let mut boxes = Vec::new();
                for i in 0..100 {
                    boxes.push(MSBox::new(&a, (tid, i)));
                }
                boxes
            }));
        }

        for h in handles {
            let boxes = h.join().unwrap();
            for (tid, i) in boxes.iter().map(|b| **b) {
                assert!(tid < 8);
                assert!(i < 100);
            }
        }
    }

    #[test]
    fn msbox_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<MSBox<i32>>();
    }

    #[test]
    fn msbox_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<MSBox<i32>>();
    }

    #[test]
    fn allocator_default() {
        ms_init_once();
        let allocator = MSAllocator::default();
        let b = MSBox::new(&allocator, "hello");
        assert_eq!(*b, "hello");
    }
}
