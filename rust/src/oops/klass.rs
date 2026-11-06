use crate::{class_loader::class_loader_data::ClassLoaderData, oops::{method::Method, normal_klass::NormalKlass, symbol_table::SymbolHandle}};

#[derive(Debug)]
pub enum Klass<'cld> {
    Normal(NormalKlass<'cld>),
    Primitive,
    Array,
}

impl<'dc> Klass<'dc> {
    pub fn as_normal(&self) -> Option<&NormalKlass<'dc>> {
        if let Klass::Normal(klass) = self {
            Some(klass)
        } else {
            None
        }
    }
}

impl<'this> Klass<'this> {
    pub fn name(&self) -> &SymbolHandle {
        if let Klass::Normal(klass) = self {
            return klass.name();
        }

        unreachable!("Unsupported klass variance")
    }

    pub fn def_cld(&self) -> &'this ClassLoaderData {
        if let Klass::Normal(klass) = self {
            return klass.def_cld;
        }

        // todo: return bootstrap here
        unreachable!("Unsupported klass variance")
    }

    pub fn super_klass(&self) -> Option<&NormalKlass<'this>> {
        if let Klass::Normal(klass) = self {
            return klass.super_klass;
        }

        // todo: return java.lang.Object here
        unreachable!("Unsupported klass variance")
    }
}
