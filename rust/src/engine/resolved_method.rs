use crate::{class_loader::ms_api::MSRef, oops::{method::Method, normal_klass::NormalKlass}};

#[derive(Debug)]
pub struct ResolvedMethod {
    holder: MSRef<NormalKlass>,
    method: MSRef<Method>
}

impl ResolvedMethod {
    pub fn new(
        holder: MSRef<NormalKlass>,
        method: MSRef<Method>,
    ) -> Self {
        Self { holder, method }
    }

    pub fn holder(&self) -> &NormalKlass {
        &self.holder
    }

    pub fn method(&self) -> &Method {
        &self.method
    }
}
