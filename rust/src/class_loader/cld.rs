use crate::{class_loader::ms_box::{MSAllocator, MSBox}, oops::{klass::Klass, oop_handle::OOPHandle}};

const CLD_MIRROR_STORAGE: i32 = 1;

struct ClassLoaderData {
    next: *mut ClassLoaderData,
    
    mirror: OOPHandle,
    
    ms_allocator: MSAllocator,
}
