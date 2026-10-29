use std::cell::OnceCell;

use crate::{
    class_loader::{load_error::LoadResult, oop_storage::OOPStorage}, oops::{klass::Klass, symbol_table::SymbolHandle}, runtime::ms_api::MsAllocator
};

#[derive(Debug)]
pub struct ClassLoaderData<'cld> {
    msa: MsAllocator,
    storage: OnceCell<OOPStorage<'cld>>,
}

unsafe impl Sync for ClassLoaderData<'_> {}

impl<'cld> ClassLoaderData<'cld> {
    pub fn new() -> Self {
        Self {
            msa: MsAllocator::new(),
            storage: OnceCell::new(),
        }
    }

    pub fn initialize(&'cld self) {
        self.storage.set(OOPStorage::new(&self.msa)).expect("Duplicated CLD initialize");
    }

    pub fn oop_storage(&self) -> &OOPStorage<'cld> {
        self.storage.get().expect("Uninitialized storage")
    }

    pub fn ms_allocator(&self) -> &MsAllocator {
        &self.msa
    }

    pub fn load_klass(&self, name: &SymbolHandle) -> LoadResult<&Klass<'cld>> {
        unimplemented!()
    }
}
