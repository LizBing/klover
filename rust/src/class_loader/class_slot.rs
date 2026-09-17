use crate::{oops::klass::Klass, runtime::ms_api::{MsBox, MsRef}};

pub struct ClassSlot {
    klass: MsBox<Klass>,
}

unsafe impl Send for ClassSlot {}
unsafe impl Sync for ClassSlot {}

impl ClassSlot {
    pub fn klass(&self) -> MsRef<Klass> {
        MsRef::from(&self.klass)
    }

    pub fn new(klass: MsBox<Klass>) -> Self {
        Self { klass }
    }
}
