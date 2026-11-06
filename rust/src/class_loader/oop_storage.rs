use crate::{
    gc_bindings::oop_hierarchy::AtomicNObjPtr,
    runtime::ms_api::{MsAllocator, MsBox},
};
use std::ptr::NonNull;

/// Slots live in the arena and are never individually removed or destroyed.
#[derive(Debug)]
struct SlotAddress(NonNull<AtomicNObjPtr>);

// SAFETY: the registry mutex protects these stable addresses, and the slot
// contents are atomic. The allocator outlives the registry and all handles.
unsafe impl Send for SlotAddress {}

#[derive(Debug, Clone, Copy)]
pub struct OopHandle<'storage> {
    pub slot: &'storage AtomicNObjPtr,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct WeakOopSlotId(usize);

#[derive(Debug)]
pub struct OopStorage {
    msa: &'static MsAllocator,

    // Keep the arena lifetime outside the mutable registry. Putting &'msa
    // references inside a Mutex would make OOPStorage invariant in 'msa.
    strongs: parking_lot::Mutex<Vec<SlotAddress>>,
    weaks: parking_lot::Mutex<Vec<SlotAddress>>,
}

impl OopStorage {
    pub(super) fn new(msa: &'static MsAllocator) -> Self {
        Self {
            msa,
            strongs: parking_lot::Mutex::new(Vec::new()),
            weaks: parking_lot::Mutex::new(Vec::new()),
        }
    }
}

impl OopStorage {
    fn create_handle(
        &self,
        storage: &parking_lot::Mutex<Vec<SlotAddress>>,
    ) -> (usize, OopHandle<'_>) {
        let slot = MsBox::new(self.msa, AtomicNObjPtr::new()).leak();

        let mut guard = storage.lock();
        let idx = guard.len();
        guard.push(SlotAddress(NonNull::from(slot)));

        (idx, OopHandle { slot })
    }

    /// A handle must not outlive the registry that makes it visible to the GC.
    ///
    /// Ownership is checked against the real private modules by `make test-compile`
    /// (`rust/compile/ownership.rs`), including a matching valid example.
    pub fn create_strong(&self) -> OopHandle<'_> {
        let (_, res) = self.create_handle(&self.strongs);
        res
    }

    pub fn create_weak(&self) -> OopHandle<'_> {
        let (_, res) = self.create_handle(&self.weaks);
        res
    }

    pub(super) fn create_weak_id(&self) -> WeakOopSlotId {
        let (idx, _) = self.create_handle(&self.weaks);
        WeakOopSlotId(idx)
    }

    pub(super) fn get_weak<'s>(&'s self, id: WeakOopSlotId) -> OopHandle<'s> {
        let WeakOopSlotId(idx) = id;
        let guard = self.weaks.lock();
        let slot = guard.get(idx).unwrap();

        unsafe {
            OopHandle {
                slot: slot.0.as_ref(),
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/oop_storage.rs"]
mod tests;
