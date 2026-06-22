use std::{ptr::NonNull, sync::{LazyLock, OnceLock}};

use crate::{class_parser::class_file::ClassFile, oops::{attr::Attribute, cp_entry::CPEntry, field::Field, method::Method}};

pub struct NormalKlass {
    cf: ClassFile,
    super_class: Option<NonNull<NormalKlass>>,

    cp: Vec<LazyLock<CPEntry>>,

    fields: Vec<Field>,
    methods: Vec<Method>,
}

impl NormalKlass {
}
