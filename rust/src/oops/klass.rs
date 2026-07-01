use crate::{
    class_loader::ms_box::MSBox,
    oops::{
        array_klass::ArrayKlass, normal_klass::NormalKlass, prim_klass::PrimKlass,
        symbol_table::SymbolHandle,
    },
};

pub enum Klass {
    Normal(NormalKlass),
    Primitive(PrimKlass),
    Array(ArrayKlass),
}

impl Klass {}
