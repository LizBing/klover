use crate::oops::{array_klass::ArrayKlass, normal_klass::NormalKlass, prim_klass::PrimKlass};

pub enum Klass {
    Normal(NormalKlass),
    Primitive(PrimKlass),
    Array(ArrayKlass)
}

impl Klass {}
