use std::ptr::null_mut;

use crate::{class_loader::ms_box::{MSAllocator, MSBox, ms_init}, oops::{klass::Klass, normal_klass::NormalKlass, oop_handle::{CLD_MIRROR_STORAGE_ID, OOPHandle}}};

pub struct ClassLoaderData {
    next: *mut ClassLoaderData,

    mirror: OOPHandle,

    pub ms_allocator: MSAllocator,

    klasses: Vec<MSBox<Klass>>,
}

impl ClassLoaderData {
    pub fn new_phase1_test_cld() -> Self {
        Self {
            next: null_mut(),
            mirror: OOPHandle::new(CLD_MIRROR_STORAGE_ID),
            ms_allocator: MSAllocator::new(),
            klasses: Vec::new()
        }
    }
}
