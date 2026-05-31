// Ordinary Object Pointer

use std::ptr::NonNull;

#[repr(C)]
pub(crate) struct ObjDesc {
    pub(crate) markword: u64,
    pub(crate) payload: [u8; 0],
}

pub(crate) type OOP = *mut ObjDesc;
pub(crate) type NarrowOOP = u32;

/* -------------------------------------------------------------------------- */
/*  FFI to libjvm (core/gc/oop_storage.c)                                     */
/* -------------------------------------------------------------------------- */

unsafe extern "C" {
    fn alloc_oop_slot(storage_id: i32) -> *mut OOP;
    fn free_oop_slot(storage_id: i32, slot: *mut OOP);
}

/* -------------------------------------------------------------------------- */
/*  OOPHandle                                                                 */
/*                                                                           */
/*  An OOPHandle is the Rust-side mechanism for native code to hold a         */
/*  reference to a Java heap object.  It wraps an OopSlot allocated from      */
/*  the C OopStorage so that the GC can iterate the slot as a root.           */
/*                                                                           */
/*  Memory model:                                                             */
/*    OOPHandle.slot --> [oop slot in OopStorage] --> ObjDesc (Java heap)    */
/*                        (oop_t = ObjDesc*)                                  */
/* -------------------------------------------------------------------------- */

pub struct OOPHandle {
    slot: NonNull<OOP>,
    storage_id: i32,
}

impl OOPHandle {
    /// Allocate a new slot from the given OopStorage.
    ///
    /// The slot is initially NULL.  Use `replace` to store an object reference.
    /// Returns `None` on allocation failure (OOM).
    pub fn new(storage_id: i32) -> Option<Self> {
        let raw = unsafe { alloc_oop_slot(storage_id) };
        NonNull::new(raw).map(|slot| Self { slot, storage_id })
    }

    /// Read the oop currently stored in the slot.
    pub fn resolve(&self) -> OOP {
        unsafe { *self.slot.as_ptr() }
    }

    /// Write a new oop value into the slot.
    pub fn replace(&self, oop: OOP) {
        unsafe {
            *self.slot.as_ptr() = oop;
        }
    }
}

impl Drop for OOPHandle {
    fn drop(&mut self) {
        unsafe { free_oop_slot(self.storage_id, self.slot.as_ptr()) };
    }
}
