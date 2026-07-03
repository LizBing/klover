use std::marker::PhantomData;

use crate::oops::{oop_handle::{KLASS_OOP_STORAGE_ID, OOPHandle}, symbol_table::SymbolHandle};

#[derive(Debug)]
pub struct PrimKlass {
    __: PhantomData<()>,
    
    pub name: SymbolHandle,
    pub size: usize,
    pub mirror: OOPHandle,
}

impl PrimKlass {
    pub fn new(name: &str, size: usize) -> Self {
        Self {
            __: PhantomData,

            name: name.into(),
            size: size,
            mirror: OOPHandle::new(KLASS_OOP_STORAGE_ID)
        }
    }
}
