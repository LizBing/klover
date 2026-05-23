use std::ptr::NonNull;

use crate::class_loader::symbol_table::Symbol;

pub struct SymbolHandle {
    pub(super) symbol: NonNull<Symbol>
}

impl Clone for SymbolHandle {
    fn clone(&self) -> Self {
        unsafe { self.symbol.as_ref().inc_ref_cnt(); }
        Self { symbol: self.symbol }
    }
}

impl Drop for SymbolHandle {
    fn drop(&mut self) {
        unsafe { self.symbol.as_ref().dec_ref_cnt(); }
    }
}

impl SymbolHandle {
    pub fn equals(&self, n: &Self) -> bool {
        self.symbol == n.symbol
    }

    pub fn utf8(&self) -> &String {
        unsafe { self.symbol.as_ref().utf8() }
    }
}
