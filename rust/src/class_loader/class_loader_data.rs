use crate::{class_loader::{load_error::LoadResult, ms_api::{MSAllocator, MSRef}}, oops::klass::Klass};

pub struct ClassLoaderData {
    pub msa: MSAllocator,
}

impl ClassLoaderData {
    pub fn load_class(&self, name: &str) -> LoadResult<MSRef<Klass>> {
        unimplemented!()
    }
}
