use std::{ffi::c_void, ptr::null_mut, sync::atomic::{AtomicU32, Ordering}};

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
}

#[repr(transparent)]
#[derive(Debug)]
pub struct AtomicNObjPtr(AtomicU32);

impl AtomicNObjPtr {
    pub fn new() -> Self {
        Self(AtomicU32::new(0))
    }

    pub fn load<const VOLATILE: bool>(&self) -> NObjPtr {
        let Self(atom) = self;
        if VOLATILE {
            NObjPtr(atom.load(Ordering::Relaxed))
        } else {
            unsafe { NObjPtr(*atom.as_ptr()) }
        }
    }

    pub fn store<const VOLATILE: bool>(&self, n: NObjPtr) {
        let Self(atom) = self;
        let NObjPtr(value) = n;
        if VOLATILE {
            atom.store(value, Ordering::Relaxed)
        } else {
            unsafe { *atom.as_ptr() = value }
        }
    }
}
