use std::{mem, ptr};
use std::ptr::{NonNull, drop_in_place};
use std::sync::atomic::{AtomicUsize, Ordering};

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
    pub fn new() -> Self {
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
        unsafe { ptr::drop_in_place(self.raw.as_ptr()); }
    }
}

// SAFETY: MSBox owns a uniquely-allocated region of metaspace memory,
// so it is Send/Sync under the same conditions as Box<T>.
unsafe impl<T: Send> Send for MSBox<T> {}
unsafe impl<T: Sync> Sync for MSBox<T> {}
