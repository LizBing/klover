use crate::{
    class_loader::{load_error::LoadResult, oop_storage::OopStorage}, oops::{klass::Klass, symbol_table::SymbolHandle}, runtime::ms_api::MsAllocator
};

#[derive(Debug)]
pub struct ClassLoaderData {
    msa: &'static MsAllocator,
    storage: OopStorage,
}

// unsafe impl Sync for ClassLoaderData<'_> {}

impl Drop for ClassLoaderData {
    fn drop(&mut self) {
        unsafe {
            Box::from_raw(self.msa as *const MsAllocator as *mut MsAllocator);
        }
    }
}

impl ClassLoaderData {
    pub fn new() -> Self {
        let msa = Box::leak(Box::new(MsAllocator::new()));
        
        Self {
            msa,
            storage: OopStorage::new(msa),
        }
    }

    pub fn oop_storage(&self) -> &OopStorage {
        &self.storage
    }

    pub fn ms_allocator(&self) -> &MsAllocator {
        self.msa
    }

    pub fn load_klass<'this>(&'this self, name: &SymbolHandle) -> LoadResult<&Klass<'this>> {
        unimplemented!()
    }
}
