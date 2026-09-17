use crate::{class_loader::load_error::LoadResult, oops::klass::Klass, runtime::ms_api::{MsAllocator, MsRef}};


pub struct ClassLoaderData {
    msa: MsAllocator,
}

impl ClassLoaderData {
    pub fn ms_allocator(&self) -> &MsAllocator {
        &self.msa
    }

    pub fn load_class(&self, name: &str) -> LoadResult<MsRef<Klass>> {
        unimplemented!()
    }
}
