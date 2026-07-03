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
    pub fn init_oop_storages();
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

#[derive(Debug)]
pub struct OOPHandle {
    slot: NonNull<OOP>,
    storage_id: i32,
}

impl OOPHandle {
    /// Allocate a new slot from the given OopStorage.
    ///
    /// The slot is initially NULL.  Use `replace` to store an object reference.
    ///
    /// # Panics
    ///
    /// The underlying C allocator aborts the process on OOM, so this function
    /// never returns in out-of-memory situations.
    pub fn new(storage_id: i32) -> Self {
        let raw = unsafe { alloc_oop_slot(storage_id) };
        // alloc_oop_slot aborts on OOM, never returns NULL.
        let slot = NonNull::new(raw).expect("alloc_oop_slot returned NULL");
        Self { slot, storage_id }
    }
}

impl Drop for OOPHandle {
    fn drop(&mut self) {
        unsafe { free_oop_slot(self.storage_id, self.slot.as_ptr()) };
    }
}

pub const KLASS_OOP_STORAGE_ID: i32 = 0;
pub const CLD_MIRROR_STORAGE_ID: i32 = 1;
