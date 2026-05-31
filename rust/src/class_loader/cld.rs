use std::ptr::NonNull;

use crate::{class_loader::ms_box::MSAllocator, oops::oop_handle::OOPHandle};

const CLD_MIRROR_STORAGE: i32 = 1;

struct ClassLoaderData {
    next: *mut ClassLoaderData,
    prev: *mut ClassLoaderData,
    
    ms_allocator: MSAllocator,

    mirror: OOPHandle
}
