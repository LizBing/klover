use std::{ptr::NonNull, sync::OnceLock};

use parking_lot::Mutex;

use crate::runtime::arguments::Arguments;

#[repr(transparent)]
struct VirtSpace(std::ffi::c_void);

unsafe extern "C" {
    fn create_virt_space(base: usize, size: usize, exec: bool) -> *mut VirtSpace;
    fn destroy_virt_space(vs: *mut VirtSpace);
    fn vs_expand_by(vs: *mut VirtSpace, size: usize, touch: bool) -> usize;
}

pub const MSCHUNK_UNIT_SIZE: usize = 8 * 1024;
pub const METASPACE_BASE: usize = 1usize << 43;
pub const METASPACE_SIZE: usize = 32 * 1024 * 1024 * 1024;

/// An owned region with a fixed starting address and unit count.
/// Moving it transfers responsibility for releasing the same memory range.
#[derive(Debug)]
pub struct MsChunk {
    start: usize,
    unit_count: usize,
    _private: (),
}

impl MsChunk {
    pub fn start(&self) -> usize {
        self.start
    }

    pub fn unit_count(&self) -> usize {
        self.unit_count
    }

    pub fn byte_size(&self) -> usize {
        self.unit_count * MSCHUNK_UNIT_SIZE
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsInitError {
    VSpaceFailed,
    AlreadyInitialized,
}

static METASPACE: OnceLock<Metaspace> = OnceLock::new();

pub struct Metaspace {
    vs: Mutex<NonNull<VirtSpace>>,
    // A synchronized list supports multiple consumers and reuses sized chunks too.
    free_list: Mutex<Vec<MsChunk>>,
}

unsafe impl Send for Metaspace {}
unsafe impl Sync for Metaspace {}

impl Drop for Metaspace {
    fn drop(&mut self) {
        unsafe { destroy_virt_space(self.vs.get_mut().as_ptr()); }
    }
}

impl Metaspace {
    fn try_new(base: usize, size: usize) -> Result<Self, MsInitError> {
        let vs = NonNull::new(unsafe { create_virt_space(base, size, false) })
            .ok_or(MsInitError::VSpaceFailed)?;

        // Leave the first unit unused: compressed pointer zero is the null sentinel.
        // Keeping chunk starts aligned to 8 KB also simplifies arena alignment.
        if unsafe { vs_expand_by(vs.as_ptr(), MSCHUNK_UNIT_SIZE, false) } == 0 {
            return Err(MsInitError::VSpaceFailed);
        }

        Ok(Self {
            vs: Mutex::new(vs),
            free_list: Mutex::new(Vec::new()),
        })
    }

    pub fn initialize() -> Result<(), MsInitError> {
        let metaspace = Self::try_new(METASPACE_BASE, METASPACE_SIZE)?;
        
        METASPACE.set(metaspace)
            .map_err(|_| MsInitError::AlreadyInitialized)
    }

    fn global() -> &'static Metaspace {
        METASPACE.get().unwrap()
    }

    pub fn release_chunk(chunk: MsChunk) {
        Self::global()
            .free_list
            .lock()
            .push(chunk);
    }

    fn take_free_chunk(&self, count: usize) -> Option<MsChunk> {
        let mut free = self.free_list.lock();
        let index = free.iter().rposition(|chunk| chunk.unit_count >= count)?;
        Some(free.remove(index))
    }

    pub fn alloc_one_chunk_fast_path() -> Option<MsChunk> {
        Self::global()
            .take_free_chunk(1)
    }

    fn try_alloc_chunk(&self, count: usize, touch: bool) -> Option<MsChunk> {
        let bytes = MSCHUNK_UNIT_SIZE
            .checked_mul(count)
            .filter(|size| *size > 0)?;
        if let Some(chunk) = self.take_free_chunk(count) {
            return Some(chunk);
        }
        let guard = self.vs.lock();
        let start = unsafe { vs_expand_by(guard.as_ptr(), bytes, touch) };
        if start == 0 {
            return None;
        }
        Some(MsChunk {
            start,
            unit_count: count,
            _private: (),
        })
    }

    pub fn alloc_chunk(count: usize) -> MsChunk {
        Self::global()
            .try_alloc_chunk(count, Arguments::ms_pretouch())
            .expect("invalid chunk count or out of metaspace memory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_failure_is_reported() {
        // A fixed address must be page aligned; this fails without replacing a mapping.
        assert!(matches!(
            Metaspace::try_new(1, MSCHUNK_UNIT_SIZE),
            Err(MsInitError::VSpaceFailed)
        ));
    }

    #[test]
    fn chunks_are_distinct_and_exhaustion_does_not_advance_top() {
        let ms = Metaspace::try_new(0, 4 * MSCHUNK_UNIT_SIZE).unwrap();
        let first = ms.try_alloc_chunk(1, true).unwrap();
        assert!(ms.try_alloc_chunk(3, false).is_none());
        let second = ms.try_alloc_chunk(2, false).unwrap();
        assert_eq!(first.unit_count(), 1);
        assert_eq!(second.start(), first.start() + first.byte_size());
        assert_eq!(second.unit_count(), 2);
        assert_eq!(second.byte_size(), 2 * MSCHUNK_UNIT_SIZE);
        assert!(ms.try_alloc_chunk(1, false).is_none());
        unsafe {
            std::ptr::write_bytes(first.start() as *mut u8, 0xAB, first.byte_size());
            std::ptr::write_bytes(second.start() as *mut u8, 0xCD, second.byte_size());
        }
    }

    #[test]
    fn released_chunks_keep_their_memory_range_and_are_reused_whole() {
        let ms = Metaspace::try_new(0, 4 * MSCHUNK_UNIT_SIZE).unwrap();
        let chunk = ms.try_alloc_chunk(3, false).unwrap();
        let start = chunk.start();
        ms.free_list.lock().push(chunk);
        // A smaller request reuses the entire region without splitting it.
        let reused = ms.try_alloc_chunk(1, false).unwrap();
        assert_eq!(reused.start(), start);
        assert_eq!(reused.unit_count(), 3);
        assert_eq!(reused.byte_size(), 3 * MSCHUNK_UNIT_SIZE);
        assert!(ms.free_list.lock().is_empty());
    }

    #[test]
    fn recycling_selects_chunks_with_enough_units_in_lifo_order() {
        let ms = Metaspace::try_new(0, 8 * MSCHUNK_UNIT_SIZE).unwrap();
        let small = ms.try_alloc_chunk(1, false).unwrap();
        let medium = ms.try_alloc_chunk(2, false).unwrap();
        let large = ms.try_alloc_chunk(3, false).unwrap();
        let starts = [small.start, medium.start, large.start];
        ms.free_list.lock().extend([large, small, medium]);

        assert!(ms.take_free_chunk(4).is_none());
        for (requested, index, actual) in [(2, 1, 2), (2, 2, 3), (1, 0, 1)] {
            let chunk = ms.take_free_chunk(requested).unwrap();
            assert_eq!(chunk.start(), starts[index]);
            assert_eq!(chunk.unit_count(), actual);
            assert_eq!(chunk.byte_size(), actual * MSCHUNK_UNIT_SIZE);
        }
        assert!(ms.free_list.lock().is_empty());
    }

    #[test]
    fn insufficient_free_chunks_cannot_satisfy_a_larger_request() {
        let ms = Metaspace::try_new(0, 4 * MSCHUNK_UNIT_SIZE).unwrap();
        let small = ms.try_alloc_chunk(1, false).unwrap();
        let medium = ms.try_alloc_chunk(2, false).unwrap();
        let small_start = small.start();
        let medium_start = medium.start();
        ms.free_list.lock().extend([medium, small]);

        assert!(ms.try_alloc_chunk(3, false).is_none());
        let reused = ms.try_alloc_chunk(2, false).unwrap();
        assert_eq!(reused.start(), medium_start);
        assert_eq!(reused.unit_count(), 2);
        let reused = ms.try_alloc_chunk(1, false).unwrap();
        assert_eq!(reused.start(), small_start);
        assert_eq!(reused.unit_count(), 1);
    }

    #[test]
    fn invalid_counts_leave_capacity_available() {
        let ms = Metaspace::try_new(0, 2 * MSCHUNK_UNIT_SIZE).unwrap();
        assert!(ms.try_alloc_chunk(0, false).is_none());
        assert!(ms.try_alloc_chunk(usize::MAX, false).is_none());
        assert!(ms.try_alloc_chunk(1, false).is_some());
    }

    #[test]
    fn concurrent_recycling_never_returns_the_same_live_chunk() {
        let ms = Metaspace::try_new(0, 9 * MSCHUNK_UNIT_SIZE).unwrap();
        let chunks: Vec<_> = (0..8)
            .map(|_| ms.try_alloc_chunk(1, false).unwrap())
            .collect();
        ms.free_list.lock().extend(chunks);
        let barrier = std::sync::Barrier::new(8);
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    scope.spawn(|| {
                        let chunk = ms.try_alloc_chunk(1, false).unwrap();
                        barrier.wait();
                        chunk
                    })
                })
                .collect();
            let mut starts: Vec<_> = handles
                .into_iter()
                .map(|h| h.join().unwrap().start)
                .collect();
            starts.sort_unstable();
            starts.dedup();
            assert_eq!(starts.len(), 8);
        });
    }
}
