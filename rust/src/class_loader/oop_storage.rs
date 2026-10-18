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
    /// ```compile_fail
    /// use klover::{class_loader::oop_storage::OOPStorage, runtime::ms_api::MsAllocator};
    /// let arena = MsAllocator::new();
    /// let storage = OOPStorage::new(&arena);
    /// let handle = storage.create_strong();
    /// drop(storage);
    /// println!("{handle:?}");
    /// ```
    pub fn create_strong(&self) -> OOPHandle<'_> {
        self.create_handle(&self.strongs)
    }

    pub fn create_weak(&self) -> OOPHandle<'_> {
        self.create_handle(&self.weaks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_survive_registry_growth_and_remain_registered() {
        let arena = MsAllocator::new();
        let storage = OOPStorage::new(&arena);
        let first = storage.create_strong();
        for _ in 0..100 {
            storage.create_strong();
        }
        let weak = storage.create_weak();
        assert_eq!(storage.strongs.lock()[0].0, NonNull::from(first.slot));
        assert_eq!(storage.weaks.lock()[0].0, NonNull::from(weak.slot));
        assert!(!std::ptr::eq(first.slot, weak.slot));
    }
}
