use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{ptr, slice};

use parking_lot::{Mutex, RwLock};

const SMALL_CHUNK_BYTE_SIZE: usize = 8 * 1024; // 8 KB
const BUMP_THRESHOLD: usize = SMALL_CHUNK_BYTE_SIZE / 2; // 4 KB

// ── Metaspace 压缩指针 ────────────────────────────────────────────────
//
// 与 C 层 `core/memory/comp_space_defs.h` 保持一致。
//
//   base = METASPACE_BASE = 1 << 43
//   shift = 3（8 字节对齐）
//   编码后 32 位 narrow ptr 覆盖 32 GB，与 COMPSPACE_BYTE_SIZE 相同。
//   narrow == 0 保留给 NULL。
//
// 任何 metaspace 内分配的、可被 MsRef 引用的结构（Klass、Field、Method 等）
// 都可以用这套编解码。

/// Metaspace 的虚拟内存基址（与 C 层 `METASPACE_BASE` 一致）。
const METASPACE_BASE: usize = 1usize << 43;

/// 压缩指针的对齐粒度（与 C 层 `COMP_PTR_SHIFT` 一致）。
const COMP_PTR_SHIFT: u32 = 3;

/// 将一个 metaspace 内的裸指针编码为 32 位 narrow ptr。
///
/// `ptr` 为 null 时返回 0。
///
/// # Panics
/// `ptr` 必须位于 `[METASPACE_BASE, METASPACE_BASE + 32GB)` 内且 8 字节对齐，
/// 否则 panic（编码失败说明分配器或调用方有 bug）。
fn ms_comp_ptr_encode<T>(ptr: *const T) -> u32 {
    if ptr.is_null() {
        return 0;
    }
    let addr = ptr as usize;
    assert!(
        addr >= METASPACE_BASE,
        "ms_comp_ptr_encode: ptr {:#x} below METASPACE_BASE",
        addr
    );
    let off = addr - METASPACE_BASE;
    assert!(
        off >> COMP_PTR_SHIFT <= u32::MAX as usize,
        "ms_comp_ptr_encode: offset overflow"
    );
    let narrow = (off >> COMP_PTR_SHIFT) as u32;
    narrow
}

/// 将 32 位 narrow ptr 解码为 metaspace 内的裸指针。
///
/// `narrow == 0` 返回 null。
fn ms_comp_ptr_decode<T>(narrow: u32) -> *mut T {
    if narrow == 0 {
        return std::ptr::null_mut();
    }
    let addr = METASPACE_BASE + ((narrow as usize) << COMP_PTR_SHIFT);
    addr as *mut T
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MsInitStatus {
    Initialized,
    AlreadyInitialized,
    VSpaceFailed,
}

impl TryFrom<i32> for MsInitStatus {
    type Error = i32;

    fn try_from(raw: i32) -> Result<Self, Self::Error> {
        match raw {
            0 => Ok(Self::Initialized),
            1 => Ok(Self::AlreadyInitialized),
            2 => Ok(Self::VSpaceFailed),
            other => Err(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsInitError {
    VSpaceFailed,
    UnknownStatus(i32),
}

unsafe extern "C" {
    fn c_ms_try_init() -> i32;
    fn ms_alloc_small_chunk() -> *mut MSChunk;
    fn ms_alloc_sized_chunk(byte_size: usize) -> *mut MSChunk;
    fn ms_free_chunk(chunk: *mut MSChunk);
}

/// Initialize the process-wide metaspace once. Failures are cached as well:
/// callers must not retry a failed native initialization implicitly.
pub fn ensure_initialized() -> Result<(), MsInitError> {
    static INIT: std::sync::OnceLock<Result<(), MsInitError>> = std::sync::OnceLock::new();
    *INIT.get_or_init(|| {
        // SAFETY: all Rust initialization calls are serialized by INIT.
        let raw = unsafe { c_ms_try_init() };
        match MsInitStatus::try_from(raw) {
            Ok(MsInitStatus::Initialized | MsInitStatus::AlreadyInitialized) => Ok(()),
            Ok(MsInitStatus::VSpaceFailed) => Err(MsInitError::VSpaceFailed),
            Err(raw) => Err(MsInitError::UnknownStatus(raw)),
        }
    })
}

#[repr(C)]
struct MSChunk {
    _next: *mut MSChunk,
    byte_size: usize,
    start: usize,
}

// SAFETY: chunks are allocated by the C metaspace layer and protected
// by internal synchronisation (Mutex / RwLock) in MsAllocator.
unsafe impl Send for MSChunk {}
unsafe impl Sync for MSChunk {}

pub struct MsAllocator {
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
unsafe impl Send for MsAllocator {}
unsafe impl Sync for MsAllocator {}

impl MsAllocator {
    pub const fn new() -> Self {
        MsAllocator {
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
    fn alloc<T>(&self) -> &mut MaybeUninit<T> {
        let size = size_of::<T>();

        unsafe {
            if size < BUMP_THRESHOLD {
                &mut *self.bump_alloc(size)
            } else {
                &mut *self.sized_alloc(size)
            }
        }
    }

    pub fn calloc<T>(&self, count: usize) -> &mut [MaybeUninit<T>] {
        let size = size_of::<T>() * count;
        let mem = if size < BUMP_THRESHOLD {
            self.bump_alloc(size)
        } else {
            self.sized_alloc(size)
        };
        let slice = unsafe { slice::from_raw_parts_mut(mem, count) };

        slice
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

impl Drop for MsAllocator {
    fn drop(&mut self) {
        // All chunks — including the current bump chunk — are tracked
        // in `chunks`.  Drain and free each one through the C layer.
        let chunks = self.chunks.get_mut();
        for chunk in chunks.drain(..) {
            unsafe { ms_free_chunk(chunk.as_ptr()) };
        }
    }
}

impl Default for MsAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// ── MsBox ─────────────────────────────────────────────────────────────

/// A pointer type that owns a heap allocation inside an
/// [`MsAllocator`]'s metaspace arena.
///
/// Individual deallocations are not supported (bump-allocator
/// semantics); memory is reclaimed when the underlying chunks are
/// destroyed together with the allocator.
#[derive(Debug)]
pub struct MsBox<T: ?Sized> {
    raw: NonNull<T>,
}

impl<T> MsBox<T> {
    /// Allocate memory through `allocator` and move `value` into it.
    pub fn new(allocator: &MsAllocator, value: T) -> Self {
        let uninit = allocator.alloc::<T>();
        let ptr = uninit.write(value);
        MsBox {
            raw: unsafe { NonNull::new_unchecked(ptr) },
        }
    }
}

impl<T: ?Sized> MsBox<T> {
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        Self {
            raw: NonNull::new(raw).unwrap(),
        }
    }
}

impl<T: ?Sized> MsBox<T> {
    /// Transfer the value to an arena reference without running its destructor.
    /// The reference remains valid only while the allocator is alive.
    pub fn leak(self) -> MsRef<T> {
        let owner = std::mem::ManuallyDrop::new(self);
        MsRef { raw: owner.raw }
    }
}

impl<T: ?Sized> Deref for MsBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.raw.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for MsBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.raw.as_mut() }
    }
}

impl<T: ?Sized> Drop for MsBox<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.raw.as_ptr());
        }
    }
}

// SAFETY: MsBox owns a uniquely-allocated region of metaspace memory,
// so it is Send/Sync under the same conditions as Box<T>.
unsafe impl<T: Send> Send for MsBox<T> {}
unsafe impl<T: Sync> Sync for MsBox<T> {}

// Safety: guaranteed by developer.
#[derive(Debug)]
pub struct MsRef<T: ?Sized> {
    raw: NonNull<T>,
}

impl<T: ?Sized> Clone for MsRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for MsRef<T> {}

/// Metaspace 压缩指针。  32 位偏移量（以 `METASPACE_BASE` 为基准，
/// 8 字节对齐）。  0 保留给 null。
pub type CompPtr = u32;

impl<T> MsRef<T> {
    /// 将引用编码为压缩指针。
    pub fn encode(&self) -> CompPtr {
        ms_comp_ptr_encode(self.raw.as_ptr())
    }

    /// 将压缩指针解码为引用。
    ///
    /// # Safety
    /// `cp` 必须是之前由 `encode` 或 `ms_comp_ptr_encode` 生成的合法值，
    /// 指向 metaspace 内类型为 `T` 的对象。
    pub unsafe fn decode(cp: CompPtr) -> Option<Self> {
        if cp == 0 {
            return None;
        }

        let ptr = ms_comp_ptr_decode::<T>(cp);
        // SAFETY: 由调用方保证 cp 合法。
        Some(Self {
            raw: unsafe { NonNull::new_unchecked(ptr) },
        })
    }

    /// 判断两个 MsRef 是否指向同一个对象（指针相等）。
    pub fn equals<U>(&self, other: &MsRef<U>) -> bool {
        self.raw.as_ptr() as *const () == other.raw.as_ptr() as *const ()
    }

    pub unsafe fn from_raw(ptr: NonNull<T>) -> Self {
        Self { raw: ptr }
    }
}

impl<T> From<&MsBox<T>> for MsRef<T> {
    fn from(value: &MsBox<T>) -> Self {
        Self { raw: value.raw }
    }
}

impl<T: ?Sized> Deref for MsRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }
    }
}

// ── tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_status_rejects_unknown_values() {
        assert_eq!(MsInitStatus::try_from(0), Ok(MsInitStatus::Initialized));
        assert_eq!(
            MsInitStatus::try_from(1),
            Ok(MsInitStatus::AlreadyInitialized)
        );
        assert_eq!(MsInitStatus::try_from(2), Ok(MsInitStatus::VSpaceFailed));
        assert_eq!(MsInitStatus::try_from(-1), Err(-1));
        assert_eq!(MsInitStatus::try_from(99), Err(99));
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
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let b = MsBox::new(&allocator, Point { x: 10, y: 20 });
        assert_eq!(b.x, 10);
        assert_eq!(b.y, 20);
    }

    #[test]
    fn alloc_int() {
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let b = MsBox::new(&allocator, 42u32);
        assert_eq!(*b, 42);
    }

    #[test]
    fn alloc_empty_tuple() {
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let b = MsBox::new(&allocator, ());
        let _ = *b;
    }

    #[test]
    fn alloc_multiple_distinct_addresses() {
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let a = MsBox::new(&allocator, 1u64);
        let b = MsBox::new(&allocator, 2u64);
        let c = MsBox::new(&allocator, 3u64);

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
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let boxes: Vec<MsBox<u32>> = (0..1000).map(|i| MsBox::new(&allocator, i)).collect();
        for (i, b) in boxes.iter().enumerate() {
            assert_eq!(**b, i as u32);
        }
    }

    #[test]
    fn deref_mut_field() {
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let mut b = MsBox::new(&allocator, Point { x: 0, y: 0 });
        b.x = 100;
        b.y = 200;
        assert_eq!(b.x, 100);
        assert_eq!(b.y, 200);
    }

    #[test]
    fn alloc_large_object() {
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let mut b = MsBox::new(&allocator, [0u8; 5 * 1024]);
        for (i, byte) in b.iter_mut().enumerate() {
            *byte = (i & 0xff) as u8;
        }
        for (i, &byte) in b.iter().enumerate() {
            assert_eq!(byte, (i & 0xff) as u8);
        }
    }

    #[test]
    fn alloc_mixed_small_and_large() {
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let small = MsBox::new(&allocator, 7u64);
        let large = MsBox::new(&allocator, [0xffu8; 5000]);
        let another = MsBox::new(&allocator, 42i32);

        assert_eq!(*small, 7);
        assert_eq!(large[0], 0xff);
        assert_eq!(large[4999], 0xff);
        assert_eq!(*another, 42);
    }

    #[test]
    fn chunk_overflow_forces_new_chunk() {
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let boxes: Vec<MsBox<[u8; 256]>> = (0..64)
            .map(|i| {
                let mut arr = [0u8; 256];
                arr[0] = i as u8;
                MsBox::new(&allocator, arr)
            })
            .collect();
        for (i, b) in boxes.iter().enumerate() {
            assert_eq!(b[0], i as u8);
        }
    }

    #[test]
    fn alloc_overaligned() {
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let b = MsBox::new(&allocator, OverAligned { data: [0xAA; 64] });
        let addr = &*b as *const OverAligned as usize;
        assert_eq!(addr % 64, 0);
    }

    #[test]
    fn drop_allocator_does_not_crash() {
        ensure_initialized().expect("initialize test metaspace");
        {
            let allocator = MsAllocator::new();
            let _a = MsBox::new(&allocator, 1u32);
            let _b = MsBox::new(&allocator, [0u8; 6000]);
            let _c = MsBox::new(&allocator, 3.14f64);
        }
    }

    #[test]
    fn drop_empty_allocator_does_not_crash() {
        ensure_initialized().expect("initialize test metaspace");
        {
            let _allocator = MsAllocator::new();
        }
    }

    #[test]
    fn concurrent_allocations() {
        ensure_initialized().expect("initialize test metaspace");
        use std::sync::Arc;

        let allocator = Arc::new(MsAllocator::new());
        let mut handles = Vec::new();

        for tid in 0..8 {
            let a = Arc::clone(&allocator);
            handles.push(std::thread::spawn(move || {
                let mut boxes = Vec::new();
                for i in 0..100 {
                    boxes.push(MsBox::new(&a, (tid, i)));
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
        assert_send::<MsBox<i32>>();
    }

    #[test]
    fn msbox_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<MsBox<i32>>();
    }

    #[test]
    fn allocator_default() {
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::default();
        let b = MsBox::new(&allocator, "hello");
        assert_eq!(*b, "hello");
    }

    struct DropProbe<'a> {
        drops: &'a AtomicUsize,
        text: String,
    }

    impl Drop for DropProbe<'_> {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn leak_transfers_value_without_dropping_it() {
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let drops = AtomicUsize::new(0);
        let mut value = MsBox::new(
            &allocator,
            DropProbe {
                drops: &drops,
                text: "still alive".into(),
            },
        );
        let raw = &mut *value as *mut DropProbe<'_>;
        let leaked = value.leak();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert_eq!(leaked.text, "still alive");
        unsafe {
            ptr::drop_in_place(raw);
        }
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn leak_preserves_slice_metadata_without_dropping_elements() {
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let drops = AtomicUsize::new(0);
        let storage = allocator.calloc(2);
        for (index, slot) in storage.iter_mut().enumerate() {
            slot.write(DropProbe {
                drops: &drops,
                text: format!("element {index}"),
            });
        }
        let raw = unsafe { storage.assume_init_mut() as *mut [DropProbe<'_>] };
        let leaked = unsafe { MsBox::from_raw(raw) }.leak();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert_eq!(leaked.len(), 2);
        assert_eq!(leaked[0].text, "element 0");
        assert_eq!(leaked[1].text, "element 1");
        unsafe {
            ptr::drop_in_place(raw);
        }
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn ordinary_msbox_still_drops_its_value() {
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let drops = AtomicUsize::new(0);
        {
            let _value = MsBox::new(
                &allocator,
                DropProbe {
                    drops: &drops,
                    text: "owned".into(),
                },
            );
        }
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}
