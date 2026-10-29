use std::sync::OnceLock;

use crate::{
    class_loader::{bs_cld::BootstrapCLD, class_loader_data::ClassLoaderData},
    class_parser::{MethodMemRef, MethodMemRefKind},
    oops::{
        jvalue::{JDouble, JFloat, JInt, JLong},
        method::Method,
        oops_errors::{ResolveError, ResolveResult},
        symbol_table::{SymbolHandle, SymbolTable},
    },
};

// #[derive(Debug)]
// pub struct StringConstant<'klass> {
//     raw: SymbolHandle,
//     obj: OOPHandle<'klass>,
// }

// impl<'klass> StringConstant<'klass> {
//     pub fn new(raw: &str, storage: &'klass OOPStorage<'_>) -> Self {
//         Self {
//             raw: SymbolTable::intern(raw),
//             obj: storage.create_strong(),
//         }
//     }

//     pub fn resolve(&self) -> NObjPtr {
//         unimplemented!()
//     }
// }

// #[derive(Debug)]
// pub struct ClassConstant<'klass> {
//     name: SymbolHandle,
//     resolved: OnceLock<ResolveResult<&'klass Klass<'klass>>>,
// }

// impl<'klass> ClassConstant<'klass> {
//     pub fn new(name: &str) -> Self {
//         Self {
//             name: SymbolTable::intern(name),
//             resolved: OnceLock::new(),
//         }
//     }

//     pub fn resolve(&self) -> ResolveResult<&Klass<'klass>> {
//         unimplemented!()
//     }
// }

// #[derive(Debug)]
// pub struct MethodTypeConstant {
// }

// impl MethodTypeConstant {
//     pub fn resolve(&self) -> NObjPtr {
//         unimplemented!()
//     }
// }

// #[derive(Debug)]
// pub struct MethodHandleConstant {}

// impl MethodHandleConstant {
//     pub fn resolve(&self) -> NObjPtr {
//         unimplemented!()
//     }
// }

#[derive(Debug, Clone)]
pub enum LdCLoadable /* <'klass> */ {
    Integer(JInt),
    Float(JFloat),
    // String(&'klass StringConstant<'klass>),
    // Class(&'klass ClassConstant<'klass>),
    // MethodType(&'klass MethodTypeConstant),
    // MethodHandle(&'klass MethodHandleConstant),
}

#[cfg(test)]
#[path = "../tests/method_resolution.rs"]
mod tests;

#[derive(Debug, Clone)]
pub enum LdC2Loadable {
    Long(JLong),
    Double(JDouble),
}

#[derive(Debug)]
pub struct MethodRef<'metadata> {
    is_interface: bool,
    owner: SymbolHandle,
    name: SymbolHandle,
    desc: SymbolHandle,
    resolved: OnceLock<&'metadata Method<'metadata>>,
}

impl<'metadata> MethodRef<'metadata> {
    pub fn build(mref: &MethodMemRef) -> Self {
        let is_interface = matches!(mref.kind, MethodMemRefKind::Interface);

        let owner = SymbolTable::intern(&mref.owner.name);
        let name = SymbolTable::intern(&mref.name);
        let desc = SymbolTable::intern(mref.descriptor.raw());

        Self {
            is_interface,
            owner,
            name,
            desc,
            resolved: OnceLock::new(),
        }
    }
}

impl<'cld> MethodRef<'cld> {
    pub fn resolve(
        &self,
        cld: Option<&ClassLoaderData<'cld>>,
    ) -> ResolveResult<&'cld Method<'cld>> {
        let owner = match cld {
            Some(cld) => cld.load_klass(&self.owner),
            None => BootstrapCLD::find_klass(&self.owner),
        }
        .map_err(|e| ResolveError::Load(e))?
        .as_normal()
        .unwrap();

        Ok(owner.find_declared_method(self.name.utf8(), self.desc.utf8()).unwrap())
    }
}
