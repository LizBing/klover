use std::ptr::NonNull;

use crate::class_loader::ms_box::MSAllocator;

struct ClassLoaderData {
    next: *mut ClassLoaderData,
    prev: *mut ClassLoaderData,
    
    ms_allocator: MSAllocator,
}
