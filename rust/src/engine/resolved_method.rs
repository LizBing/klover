use crate::{
    class_loader::ms_api::MSRef,
    oops::{cp_entry::ResolvedMethodRef, method::Method, normal_klass::NormalKlass},
};

#[derive(Debug, Clone)]
pub struct ResolvedMethod {
    holder: MSRef<NormalKlass>,
    method: MSRef<Method>,
}

impl From<ResolvedMethodRef> for ResolvedMethod {
    fn from(value: ResolvedMethodRef) -> Self {
        Self::new(value.holder, value.method)
    }
}

impl ResolvedMethod {
    pub fn new(holder: MSRef<NormalKlass>, method: MSRef<Method>) -> Self {
        Self { holder, method }
    }

    pub fn holder(&self) -> &NormalKlass {
        &self.holder
    }

    pub fn holder_ref(&self) -> MSRef<NormalKlass> {
        self.holder.clone()
    }

    pub fn method(&self) -> &Method {
        &self.method
    }
}
