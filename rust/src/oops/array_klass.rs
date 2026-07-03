use crate::oops::{desc::FieldDesc, oop_handle::OOPHandle, symbol_table::SymbolHandle};

#[derive(Debug)]
pub struct ArrayKlass {
    pub name: SymbolHandle,
    pub desc: FieldDesc,
    pub mirror: OOPHandle
}
