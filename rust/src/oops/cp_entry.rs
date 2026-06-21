use std::ptr::NonNull;

use crate::{class_loader::symbol_handle::SymbolHandle, oops::klass::Klass};
use super::desc::FieldType;

#[derive(Debug)]
enum CPEntry {
    Class(NonNull<Klass>),
}
