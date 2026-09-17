use crate::{class_loader::load_error::LoadResult, oops::klass::Klass, runtime::ms_api::{MsAllocator, MsRef}};


pub struct ClassLoaderData {
    pub msa: MsAllocator,
}

impl ClassLoaderData {
    pub fn load_class(&self, name: &str) -> LoadResult<MsRef<Klass>> {
        unimplemented!()
    }
}
