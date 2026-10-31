use crate::{
    gc_bindings::oop_hierarchy::AtomicNObjPtr,
    runtime::ms_api::{MsAllocator, MsBox},
};
use std::ptr::NonNull;

/// Registry entries do not own slots. The allocator borrow in OOPStorage keeps
/// their allocations alive; entries are only created from that allocator.
#[derive(Debug)]
struct SlotAddress(NonNull<AtomicNObjPtr>);

// SAFETY: slots contain only atomic state, are never individually destroyed,
// and the registry is protected by a mutex. Addresses never escape the storage.
unsafe impl Send for SlotAddress {}

#[derive(Debug)]
pub struct OOPHandle<'storage> {
    slot: &'storage AtomicNObjPtr,
}

#[derive(Debug)]
pub struct OopStorage {
    msa: NonNull<MsAllocator>,

    // Keep the arena lifetime outside the mutable registry. Putting &'msa
    // references inside a Mutex would make OOPStorage invariant in 'msa.
    strongs: parking_lot::Mutex<Vec<SlotAddress>>,
    weaks: parking_lot::Mutex<Vec<SlotAddress>>,
}

impl OopStorage {
    pub(super) fn new(msa: &'static MsAllocator) -> Self {
        Self {
            msa: NonNull::from(msa),
            strongs: parking_lot::Mutex::new(Vec::new()),
            weaks: parking_lot::Mutex::new(Vec::new()),
        }
    }
}

impl OopStorage {
    fn create_handle(&self, storage: &parking_lot::Mutex<Vec<SlotAddress>>) -> OOPHandle<'_> {
        let slot = unsafe { MsBox::new(self.msa.as_ref(), AtomicNObjPtr::new()).leak() };

        let mut guard = storage.lock();
        guard.push(SlotAddress(NonNull::from(slot)));

        OOPHandle { slot }
    }

    /// A handle must not outlive the registry that makes it visible to the GC.
    ///
    /// Ownership is checked against the real private modules by `make test-compile`
    /// (`rust/compile/ownership.rs`), including a matching valid example.
    pub fn create_strong(&self) -> OOPHandle<'_> {
        self.create_handle(&self.strongs)
    }

    pub fn create_weak(&self) -> OOPHandle<'_> {
        self.create_handle(&self.weaks)
    }
}

#[cfg(test)]
#[path = "../tests/oop_storage.rs"]
mod tests;
