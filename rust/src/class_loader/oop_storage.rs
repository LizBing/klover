use crate::{gc_bindings::oop_hierarchy::NObjPtr, runtime::ms_api::{MsAllocator, MsBox, MsRef}};

pub struct OOPHandle {
    slot: MsRef<NObjPtr>,
}

pub struct OOPStorage<'a> {
    msa: &'a MsAllocator,
    
    strongs: parking_lot::Mutex<Vec<MsBox<NObjPtr>>>,
    weaks: parking_lot::Mutex<Vec<MsBox<NObjPtr>>>,
}

impl<'a> OOPStorage<'a> {
    pub fn new(msa: &'a MsAllocator) -> Self {
        Self {
            msa,
            strongs: parking_lot::Mutex::new(Vec::new()),
            weaks: parking_lot::Mutex::new(Vec::new()),
        }
    }
}

impl OOPStorage<'_> {
    fn create_handle(msa: &MsAllocator, storage: &parking_lot::Mutex<Vec<MsBox<NObjPtr>>>) -> OOPHandle {
        let slot_box = MsBox::new(msa, NObjPtr::null());
        let slot_ref = (&slot_box).into();
        
        let mut guard = storage.lock();

        guard.push(slot_box);
        
        OOPHandle { slot: slot_ref }
    }

    pub fn create_strong(&self) -> OOPHandle {
        Self::create_handle(self.msa, &self.strongs)
    }

    pub fn create_weak(&self) -> OOPHandle {
        Self::create_handle(self.msa, &self.weaks)
    }
}
