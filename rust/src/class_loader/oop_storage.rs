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
pub struct OOPStorage<'msa> {
    msa: &'msa MsAllocator,

    // Keep the arena lifetime outside the mutable registry. Putting &'msa
    // references inside a Mutex would make OOPStorage invariant in 'msa.
    strongs: parking_lot::Mutex<Vec<SlotAddress>>,
    weaks: parking_lot::Mutex<Vec<SlotAddress>>,
}

impl<'msa> OOPStorage<'msa> {
    pub const fn new(msa: &'msa MsAllocator) -> Self {
        Self {
            msa,
            strongs: parking_lot::Mutex::new(Vec::new()),
            weaks: parking_lot::Mutex::new(Vec::new()),
        }
    }
}

impl<'msa> OOPStorage<'msa> {
    fn create_handle(&self, storage: &parking_lot::Mutex<Vec<SlotAddress>>) -> OOPHandle<'_> {
        let slot = MsBox::new(self.msa, AtomicNObjPtr::new()).leak();

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
