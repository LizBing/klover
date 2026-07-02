use crate::oops::{desc::FieldDesc, oop_handle::OOPHandle, symbol_table::SymbolHandle};

pub struct ArrayKlass {
    pub name: SymbolHandle,
    pub desc: FieldDesc,
    pub mirror: OOPHandle
}
