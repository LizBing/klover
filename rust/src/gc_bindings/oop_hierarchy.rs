use std::{
    ffi::c_void,
    ptr::null_mut,
    sync::atomic::{AtomicU32, Ordering},
};

type ObjPtrImpl = c_void;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct ObjPtr(*mut ObjPtrImpl);

impl ObjPtr {
    pub fn null() -> Self {
        Self(null_mut())
    }

    pub fn is_null(self) -> bool {
        let Self(raw) = self;
        raw.is_null()
    }

    pub fn as_usize(self) -> usize {
        let Self(raw) = self;
        raw as _
    }

    pub fn encode(self) -> NObjPtr {
        unimplemented!()
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct NObjPtr(u32);

impl NObjPtr {
    pub(crate) fn from_u32(raw: u32) -> Self {
        Self(raw)
    }
    pub fn null() -> Self {
        Self(0)
    }

    pub fn is_null(self) -> bool {
        self.as_u32() == 0
    }

    pub fn as_u32(self) -> u32 {
        let Self(raw) = self;
        raw
    }

    pub fn decode(self) -> ObjPtr {
        unimplemented!()
    }

    /// # Safety
    /// `addr` must point to initialized, readable and writable storage aligned
    /// for `AtomicU32`. Conflicting unsynchronized accesses must be atomic and
    /// cover exactly the same bytes.
    pub unsafe fn load<const VOLATILE: bool>(addr: *const Self) -> Self {
        let ordering = if VOLATILE {
            Ordering::SeqCst
        } else {
            Ordering::Relaxed
        };
        // SAFETY: NObjPtr is transparent over u32. The caller guarantees valid access.
        let atomic = unsafe { AtomicU32::from_ptr(addr.cast_mut().cast()) };
        Self(atomic.load(ordering))
    }

    /// # Safety
    /// The same storage and concurrency requirements as `load` apply.
    pub unsafe fn store<const VOLATILE: bool>(addr: *mut Self, value: NObjPtr) {
        let ordering = if VOLATILE {
            Ordering::SeqCst
        } else {
            Ordering::Relaxed
        };
        // SAFETY: NObjPtr is transparent over u32. The caller guarantees valid access.
        let atomic = unsafe { AtomicU32::from_ptr(addr.cast()) };
        atomic.store(value.as_u32(), ordering);
    }
}

/// Shared GC handle slot. Every access to the slot is atomic.
#[repr(transparent)]
#[derive(Debug)]
pub struct AtomicNObjPtr(AtomicU32);

impl AtomicNObjPtr {
    pub fn new() -> Self {
        Self(AtomicU32::new(0))
    }

    pub fn load<const VOLATILE: bool>(&self) -> NObjPtr {
        let ordering = if VOLATILE {
            Ordering::SeqCst
        } else {
            Ordering::Relaxed
        };
        NObjPtr(self.0.load(ordering))
    }

    pub fn store<const VOLATILE: bool>(&self, value: NObjPtr) {
        let ordering = if VOLATILE {
            Ordering::SeqCst
        } else {
            Ordering::Relaxed
        };
        self.0.store(value.as_u32(), ordering);
    }
}
