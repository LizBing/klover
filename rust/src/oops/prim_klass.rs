use std::marker::PhantomData;

use crate::oops::symbol_table::SymbolHandle;

#[derive(Debug)]
pub struct PrimKlass {
    __: PhantomData<()>,
    
    pub name: SymbolHandle,
    pub size: usize,
}

impl PrimKlass {
    pub fn new(name: &str, size: usize) -> Self {
        Self {
            __: PhantomData,

            name: name.into(),
            size: size,
        }
    }
}
