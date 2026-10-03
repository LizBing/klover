use crate::{
    class_loader::{load_error::LoadResult, oop_storage::OOPStorage},
    oops::normal_klass::NormalKlass,
    runtime::ms_api::MsAllocator,
};

#[derive(Debug)]
pub struct ClassLoaderData<'cld> {
    // The arena is owned outside the loader, avoiding a self-reference from
    // storage into another field of this struct.
    msa: &'cld MsAllocator,
    storage: OOPStorage<'cld>,
}

impl<'cld> ClassLoaderData<'cld> {
    pub fn new(msa: &'cld MsAllocator) -> Self {
        Self {
            msa,
            storage: OOPStorage::new(msa),
        }
    }

    pub fn oop_storage(&self) -> &OOPStorage<'cld> {
        &self.storage
    }

    pub fn ms_allocator(&self) -> &MsAllocator {
        self.msa
    }

    pub fn load_class(&'cld self, name: &str) -> LoadResult<&'cld NormalKlass<'cld>> {
        unimplemented!()
    }
}
