use crate::oops::{
        array_klass::ArrayKlass, normal_klass::NormalKlass, prim_klass::PrimKlass,
    };

#[derive(Debug)]
    pub enum Klass {
    Normal(NormalKlass),
    Primitive(PrimKlass),
    Array(ArrayKlass)
}

impl Klass {
    pub fn as_normal(&self) -> Option<&NormalKlass> {
        match self {
            Self::Normal(x) => Some(x),
            _ => None
        }
    }
}
