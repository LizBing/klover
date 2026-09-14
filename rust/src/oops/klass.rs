use std::ops::Deref;

use crate::{class_loader::ms_api::MSRef, oops::normal_klass::NormalKlass};

pub enum Klass {
    Normal(NormalKlass),
}

impl MSRef<Klass> {
    pub fn as_normal_klass_ref(self) -> Option<MSRef<NormalKlass>> {
        match self.deref() {
            Klass::Normal(nk) => unsafe { Some(MSRef::from_raw(nk.into())) }
        }
    }
}
