use crate::{class_loader::ms_api::{MSAllocator, MSBox, MSRef}, gc_bindings::oop_hierarchy::NObjPtr};

pub struct OOPHandle {
    slot: MSRef<NObjPtr>,
}

pub struct OOPStorage<'a> {
    msa: &'a MSAllocator,
    
    strongs: parking_lot::Mutex<Vec<MSBox<NObjPtr>>>,
    weaks: parking_lot::Mutex<Vec<MSBox<NObjPtr>>>,
}

impl<'a> OOPStorage<'a> {
    pub fn new(msa: &'a MSAllocator) -> Self {
        Self {
            msa,
            strongs: parking_lot::Mutex::new(Vec::new()),
            weaks: parking_lot::Mutex::new(Vec::new()),
        }
    }
}

impl OOPStorage<'_> {
    fn create_handle(msa: &MSAllocator, storage: &parking_lot::Mutex<Vec<MSBox<NObjPtr>>>) -> OOPHandle {
        let slot_box = MSBox::new(msa, NObjPtr::null());
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
