use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{ptr, slice};

use parking_lot::{Mutex, RwLock};

use crate::runtime::metaspace::{METASPACE_BASE, MSCHUNK_UNIT_SIZE, Metaspace, MsChunk};

pub use crate::runtime::metaspace::MsInitError;

/// Initialize the shared Rust metaspace; mapping failures are cached.
pub fn ensure_initialized() -> Result<(), MsInitError> {
    Metaspace::ensure_initialized()
}

const BUMP_THRESHOLD: usize = MSCHUNK_UNIT_SIZE / 2; // 4 KB

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
    assert!(off != 0, "ms_comp_ptr_encode: base is the null sentinel");
    assert_eq!(
        off & ((1 << COMP_PTR_SHIFT) - 1),
        0,
        "ms_comp_ptr_encode: pointer is not 8-byte aligned"
    );
    (off >> COMP_PTR_SHIFT) as u32
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

#[derive(Debug)]
pub struct MsAllocator {
    chunks: Mutex<Vec<MsChunk>>,
    /// Owns the current bump chunk. Retired and dedicated chunks live in `chunks`.
    /// Protected by RwLock:
    /// read-lock for the fast path (peek + CAS on offset),
    /// write-lock for the slow path (chunk swap).
    cur_chunk: RwLock<Option<MsChunk>>,
    /// Bump-pointer offset inside the current chunk.
    /// Lock-free – allocated via CAS in the fast path.
    cur_offset: AtomicUsize,
}

impl MsAllocator {
    pub const fn new() -> Self {
        MsAllocator {
            chunks: Mutex::new(Vec::new()),
            cur_chunk: RwLock::new(None),
            cur_offset: AtomicUsize::new(0),
        }
    }

    /// Small values bump within an 8 KB chunk; large values get a sized chunk.
    fn alloc<T>(&self) -> &mut MaybeUninit<T> {
        unsafe { &mut *self.alloc_raw::<MaybeUninit<T>>(size_of::<T>()) }
    }

    fn calloc<T>(&self, count: usize) -> &mut [MaybeUninit<T>] {
        let size = size_of::<T>()
            .checked_mul(count)
            .filter(|size| *size <= isize::MAX as usize)
            .expect("metaspace allocation size overflow");
        let mem = self.alloc_raw::<MaybeUninit<T>>(size);
        unsafe { slice::from_raw_parts_mut(mem, count) }
    }

    fn alloc_raw<T>(&self, size: usize) -> *mut T {
        if size == 0 {
            return NonNull::<T>::dangling().as_ptr();
        }
        if size < BUMP_THRESHOLD && std::mem::align_of::<T>() <= MSCHUNK_UNIT_SIZE {
            self.bump_alloc(size)
        } else {
            self.sized_alloc(size)
        }
    }

    fn aligned_offset<T>(chunk: &MsChunk, offset: usize) -> usize {
        // All arena addresses must also be representable as compressed pointers.
        let align = std::mem::align_of::<T>().max(1 << COMP_PTR_SHIFT);
        let address = chunk.start() + offset;
        ((address + align - 1) & !(align - 1)) - chunk.start()
    }

    fn bump_alloc<T>(&self, size: usize) -> *mut T {
        loop {
            {
                let cur = self.cur_chunk.read();
                if let Some(chunk) = cur.as_ref() {
                    let offset = self.cur_offset.load(Ordering::Relaxed);
                    let aligned = Self::aligned_offset::<T>(chunk, offset);
                    if aligned + size <= chunk.byte_size() {
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
                            return (chunk.start() + aligned) as *mut T;
                        }
                        continue;
                    }
                }
            }
            return self.bump_alloc_slow::<T>(size);
        }
    }

    fn bump_alloc_slow<T>(&self, size: usize) -> *mut T {
        let mut cur = self.cur_chunk.write();
        // The writer excludes every fast-path reader, so no CAS is needed here.
        if let Some(chunk) = cur.as_ref() {
            let offset = self.cur_offset.load(Ordering::Relaxed);
            let aligned = Self::aligned_offset::<T>(chunk, offset);
            if aligned + size <= chunk.byte_size() {
                self.cur_offset.store(aligned + size, Ordering::Relaxed);
                return (chunk.start() + aligned) as *mut T;
            }
        }

        let new_chunk =
            Metaspace::alloc_one_chunk_fast_path().unwrap_or_else(|| Metaspace::alloc_chunk(1));
        let aligned = Self::aligned_offset::<T>(&new_chunk, 0);
        let start = new_chunk.start() + aligned;
        if let Some(old_chunk) = cur.replace(new_chunk) {
            self.chunks.lock().push(old_chunk);
        }
        self.cur_offset.store(aligned + size, Ordering::Relaxed);
        start as *mut T
    }

    fn sized_alloc<T>(&self, size: usize) -> *mut T {
        let align = std::mem::align_of::<T>().max(1 << COMP_PTR_SHIFT);
        // Every chunk starts at an 8 KB boundary. Only larger alignments need padding.
        let padding = align.saturating_sub(MSCHUNK_UNIT_SIZE);
        let bytes = size
            .checked_add(padding)
            .expect("metaspace allocation size overflow");
        let count = bytes.div_ceil(MSCHUNK_UNIT_SIZE);
        let chunk = Metaspace::alloc_chunk(count);
        let start = chunk.start() + Self::aligned_offset::<T>(&chunk, 0);
        self.chunks.lock().push(chunk);
        start as *mut T
    }
}

impl Drop for MsAllocator {
    fn drop(&mut self) {
        if let Some(chunk) = self.cur_chunk.get_mut().take() {
            Metaspace::release_chunk(chunk);
        }
        for chunk in self.chunks.get_mut().drain(..) {
            Metaspace::release_chunk(chunk);
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
/// semantics); memory is recycled when the underlying chunks are
/// released together with the allocator.
#[derive(Debug)]
pub struct MsBox<'alloc, T: ?Sized> {
    _marker: PhantomData<(&'alloc MsAllocator, T)>,
    raw: NonNull<T>,
}

impl<'alloc, T> MsBox<'alloc, T> {
    /// Allocate memory through `allocator` and move `value` into it.
    pub fn new(allocator: &'alloc MsAllocator, value: T) -> Self {
        let uninit = allocator.alloc::<T>();
        let ptr = uninit.write(value);
        MsBox {
            _marker: PhantomData,
            raw: unsafe { NonNull::new_unchecked(ptr) },
        }
    }

    pub fn new_uninit_slice(
        allocator: &'alloc MsAllocator,
        count: usize,
    ) -> MsBox<'alloc, [MaybeUninit<T>]> {
        MsBox {
            _marker: PhantomData,
            raw: allocator.calloc(count).into(),
        }
    }
}

impl<'alloc, T> MsBox<'alloc, [MaybeUninit<T>]> {
    /// Transfer ownership of a fully initialized slice.
    ///
    /// # Safety
    /// Every element must contain an initialized, valid `T`.
    pub unsafe fn assume_init(self) -> MsBox<'alloc, [T]> {
        let owner = std::mem::ManuallyDrop::new(self);
        MsBox {
            _marker: PhantomData,
            // Preserve the slice length and unique ownership without creating
            // a shared reference to memory that the new owner may mutate.
            raw: unsafe { NonNull::new_unchecked(owner.raw.as_ptr() as *mut [T]) },
        }
    }
}

impl<'alloc, T: ?Sized> MsBox<'alloc, T> {
    /// Consume this owner and publish an arena-lived shared reference.
    /// Neither this value nor its owned resources will have their destructors
    /// run. The allocator eventually reclaims only the arena memory.
    ///
    /// ```compile_fail
    /// use klover::runtime::ms_api::{MsAllocator, MsBox};
    /// let arena = MsAllocator::new();
    /// let published = MsBox::new(&arena, 42).leak();
    /// drop(arena);
    /// println!("{published}");
    /// ```
    pub fn leak(self) -> &'alloc T
    where
        T: 'alloc,
    {
        let owner = std::mem::ManuallyDrop::new(self);
        unsafe { owner.raw.as_ref() }
    }
}

impl<T: ?Sized> MsBox<'_, T> {
    /// Borrow the value for no longer than this owner is borrowed.
    ///
    /// ```compile_fail
    /// use klover::runtime::ms_api::{MsAllocator, MsBox};
    /// let arena = MsAllocator::new();
    /// let owner = MsBox::new(&arena, String::from("metadata"));
    /// let borrowed = owner.share();
    /// drop(owner);
    /// println!("{borrowed}");
    /// ```
    ///
    /// ```compile_fail
    /// use klover::runtime::ms_api::{MsAllocator, MsBox};
    /// let arena = MsAllocator::new();
    /// let mut owner = MsBox::new(&arena, 1);
    /// let borrowed = owner.share();
    /// *owner = 2;
    /// println!("{borrowed}");
    /// ```
    pub fn share(&self) -> &T {
        unsafe { self.raw.as_ref() }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.raw.as_mut() }
    }
}

impl<T: ?Sized> Deref for MsBox<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.raw.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for MsBox<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.raw.as_mut() }
    }
}

impl<T: ?Sized> Drop for MsBox<'_, T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.raw.as_ptr());
        }
    }
}

// SAFETY: MsBox owns a uniquely-allocated region of metaspace memory,
// so it is Send/Sync under the same conditions as Box<T>.
unsafe impl<T: ?Sized + Send> Send for MsBox<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for MsBox<'_, T> {}

// ── tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
    fn dedicated_allocation_respects_alignment_larger_than_a_chunk() {
        #[repr(align(32768))]
        struct Aligned([u8; 32768]);
        let allocator = MsAllocator::new();
        let value = MsBox::new(&allocator, Aligned([0xAB; 32768]));
        assert_eq!((&*value as *const Aligned as usize) % 32768, 0);
        assert!(value.0.iter().all(|byte| *byte == 0xAB));
    }

    #[test]
    fn small_allocations_round_trip_through_compressed_pointers() {
        let allocator = MsAllocator::new();
        for byte in 0..32u8 {
            let value = MsBox::new(&allocator, byte);
            let pointer = &*value as *const u8;
            let narrow = ms_comp_ptr_encode(pointer);
            assert_ne!(narrow, 0);
            assert_eq!(ms_comp_ptr_decode::<u8>(narrow).cast_const(), pointer);
        }
        assert_eq!(ms_comp_ptr_encode::<u8>(ptr::null()), 0);
        assert!(ms_comp_ptr_decode::<u8>(0).is_null());
    }

    #[test]
    fn compressed_pointer_encoding_rejects_invalid_addresses() {
        for address in [
            METASPACE_BASE,
            METASPACE_BASE + 1,
            METASPACE_BASE - 8,
            METASPACE_BASE + (1usize << 35),
        ] {
            assert!(std::panic::catch_unwind(|| ms_comp_ptr_encode(address as *const u8)).is_err());
        }
    }

    #[test]
    fn empty_and_zero_sized_slices_are_supported() {
        let allocator = MsAllocator::new();
        let empty = MsBox::<u64>::new_uninit_slice(&allocator, 0);
        assert!(empty.is_empty());
        let zero_sized = MsBox::<()>::new_uninit_slice(&allocator, 8);
        assert_eq!(zero_sized.len(), 8);
    }

    #[test]
    #[should_panic(expected = "metaspace allocation size overflow")]
    fn overflowing_slice_size_is_rejected_before_allocation() {
        let allocator = MsAllocator::new();
        let _ = MsBox::<u64>::new_uninit_slice(&allocator, usize::MAX);
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

        let allocator = MsAllocator::new();
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for tid in 0..8 {
                let allocator = &allocator;
                handles.push(scope.spawn(move || {
                    (0..1000)
                        .map(|i| MsBox::new(allocator, (tid, i)))
                        .collect::<Vec<_>>()
                }));
            }
            let mut addresses = std::collections::HashSet::new();
            for (expected_tid, handle) in handles.into_iter().enumerate() {
                for (expected_i, value) in handle.join().unwrap().into_iter().enumerate() {
                    let (tid, i) = *value;
                    assert_eq!(tid, expected_tid);
                    assert_eq!(i, expected_i);
                    assert!(addresses.insert(&*value as *const _ as usize));
                }
            }
            assert_eq!(addresses.len(), 8000);
        });
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
    fn initialized_slice_can_mutate_and_drops_each_element_once() {
        ensure_initialized().expect("initialize test metaspace");
        let allocator = MsAllocator::new();
        let drops = AtomicUsize::new(0);
        let mut storage = MsBox::new_uninit_slice(&allocator, 2);
        for (index, slot) in storage.iter_mut().enumerate() {
            slot.write(DropProbe {
                drops: &drops,
                text: format!("element {index}"),
            });
        }
        let mut values = unsafe { storage.assume_init() };
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        values[1].text.push_str(" updated");
        assert_eq!(values.len(), 2);
        assert_eq!(values[1].text, "element 1 updated");
        drop(values);
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn leak_preserves_slice_metadata_and_skips_destructors() {
        struct CountDrop<'a>(&'a AtomicUsize);
        impl Drop for CountDrop<'_> {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        ensure_initialized().expect("initialize test metaspace");
        let drops = AtomicUsize::new(0);
        {
            let allocator = MsAllocator::new();
            let mut storage = MsBox::new_uninit_slice(&allocator, 2);
            for slot in storage.iter_mut() {
                slot.write(CountDrop(&drops));
            }
            let leaked = unsafe { storage.assume_init() }.leak();
            assert_eq!(leaked.len(), 2);
            assert_eq!(leaked[0].0.load(Ordering::SeqCst), 0);
        }
        // Reclaiming arena memory does not run leaked values' destructors.
        assert_eq!(drops.load(Ordering::SeqCst), 0);
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
