use std::hash::Hash;

use crate::{oops::{klass::Klass, symbol_table::SymbolHandle}, runtime::ms_api::MsBox};

pub struct ClassSlot {
    pub klass: MsBox<Klass>,
}

unsafe impl Send for ClassSlot {}
unsafe impl Sync for ClassSlot {}

impl ClassSlot {
    pub fn new(klass: MsBox<Klass>) -> Self {
        Self { klass }
    }
}
