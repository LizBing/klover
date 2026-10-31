use crate::{class_loader::class_loader_data::ClassLoaderData, oops::{method::Method, normal_klass::NormalKlass, symbol_table::SymbolHandle}};

#[derive(Debug)]
pub enum Klass<'cld> {
    Normal(NormalKlass<'cld>),
    Primitive,
    Array,
}

impl<'this> Klass<'this> {
    pub fn def_cld(&self) -> &'this ClassLoaderData {
        if let Klass::Normal(klass) = self {
            return klass.def_cld;
        }

        unreachable!("Unsupported klass variance")
    }

    pub fn find_declared_method(&self, name: &str, desc: &str) -> Option<&Method<'this>> {
        if let Klass::Normal(klass) = self {
            return klass.find_declared_method(name, desc);
        }

        unreachable!("Unsupported klass variance")
    }

    pub fn super_klass(&self) -> Option<&NormalKlass<'this>> {
        if let Klass::Normal(klass) = self {
            return klass.super_klass;
        }

        unreachable!("Unsupported klass variance")
    }
}
