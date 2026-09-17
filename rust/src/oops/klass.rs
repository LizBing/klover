use std::ops::Deref;

use crate::{runtime::ms_api::MsRef, oops::normal_klass::NormalKlass};

pub enum Klass {
    Normal(NormalKlass),
}

impl MsRef<Klass> {
    pub fn as_normal_klass_ref(self) -> Option<MsRef<NormalKlass>> {
        match self.deref() {
            Klass::Normal(nk) => unsafe { Some(MsRef::from_raw(nk.into())) }
        }
    }
}
