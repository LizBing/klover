use std::ptr::NonNull;

use crate::{class_loader::symbol_handle::SymbolHandle, class_parser::{acc_flags::AccFlags, class_file::ClassFile}};

pub struct Klass {
    name: SymbolHandle,
    super_class: Option<NonNull<Klass>>,
    acc_flags: AccFlags,
    
    instance_size: usize,

    data: KlassData,
}

enum KlassData {
    Normal(NormalKlassData),
    Primitive(PrimKlassData),
    Array(ArrayKlassData)
}

struct NormalKlassData {}

struct PrimKlassData {}

struct ArrayKlassData {}
