use crate::{class_loader::ms_api::MSRef, oops::{array_klass::ArrayKlass, normal_klass::NormalKlass, prim_klass::PrimKlass}};

#[derive(Debug)]
pub enum Klass {
    Normal(NormalKlass),
    Primitive(PrimKlass),
    Array(ArrayKlass),
}

impl Klass {
    pub fn as_normal(&self) -> Option<&NormalKlass> {
        match self {
            Self::Normal(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&ArrayKlass> {
        match self {
            Self::Array(x) => Some(x),
            _ => None,
        }
    }
}

impl MSRef<Klass> {
    pub fn as_normal_ref(&self) -> Option<MSRef<NormalKlass>> {
        let normal = self.as_normal()?;

        unsafe {
            Some(MSRef::from_raw(normal.into()))
        }
    }
}
