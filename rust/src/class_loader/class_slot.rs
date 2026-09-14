use std::hash::Hash;

use crate::{class_loader::ms_api::MSBox, oops::{klass::Klass, symbol_table::SymbolHandle}};

pub struct ClassSlot {
    pub klass: MSBox<Klass>,
}

unsafe impl Send for ClassSlot {}
unsafe impl Sync for ClassSlot {}

impl ClassSlot {
    pub fn new(klass: MSBox<Klass>) -> Self {
        Self { klass }
    }
}
