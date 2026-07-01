use std::{ptr::NonNull, sync::OnceLock};

use crate::{
    class_loader::{class_path::ClassPath, cld::ClassLoaderData, load_error::LoadResult},
    oops::klass::Klass,
};

#[derive(Debug)]
struct BootstrapPtr(*mut ClassLoaderData);
unsafe impl Send for BootstrapPtr {}
unsafe impl Sync for BootstrapPtr {}

static BOOTSTRAP_CLD: OnceLock<BootstrapPtr> = OnceLock::new();

pub fn init(class_path: Box<dyn ClassPath>) {
    let cld = ClassLoaderData::new(Some("bootstrap".into()), class_path);
    BOOTSTRAP_CLD
        .set(BootstrapPtr(cld.as_ptr()))
        .expect("bootstrap CLD already initialized");
}

pub fn cld() -> NonNull<ClassLoaderData> {
    let ptr = BOOTSTRAP_CLD.get().expect("bootstrap CLD not initialized");
    unsafe { NonNull::new_unchecked(ptr.0) }
}

pub fn load_class(name: &str) -> LoadResult<NonNull<Klass>> {
    unsafe { cld().as_ref().load_class(name) }
}
