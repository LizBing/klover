use std::sync::OnceLock;

use crate::{
    class_parser::{MethodMemRef, MethodMemRefKind},
    oops::{
        jvalue::{JDouble, JFloat, JInt, JLong},
        klass::Klass,
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

impl<'metadata> MethodRef<'metadata> {
    /// Resolve declarations and superclass methods; interface fallback is unsupported.
    /// Only successful lookups are cached. Access checks and loading constraints
    /// are not yet implemented.
    pub fn resolve(
        &self,
        caller: &Klass<'metadata>,
    ) -> ResolveResult<&'metadata Method<'metadata>> {
        if let Some(m) = self.resolved.get() {
            return Ok(*m);
        }

        let cld = caller.def_cld();
        let owner = cld.load_klass(&self.owner).map_err(ResolveError::Load)?;
        let Klass::Normal(owner) = owner else {
            unimplemented!("Method resolution for primitive and array classes")
        };

        if owner.is_interface() != self.is_interface {
            return Err(ResolveError::IncompatibleClassChange {
                owner: self.owner.utf8().into(),
                expected_interface: self.is_interface,
            });
        }

        let name = self.name.utf8();
        let desc = self.desc.utf8();
        let mut current = Some(owner);
        let mut needs_interface_lookup = self.is_interface;

        while let Some(klass) = current {
            if let Some(method) = klass.find_declared_method(name, desc) {
                // Concurrent successful lookups return the published winner.
                return Ok(*self.resolved.get_or_init(|| method));
            }

            // Interface resolution has separate Object/superinterface rules;
            // it must not use the ordinary superclass walk.
            if self.is_interface {
                break;
            }
            needs_interface_lookup |= klass.has_interfaces();
            current = klass.super_klass;
        }

        if needs_interface_lookup {
            Err(ResolveError::UnsupportedInterfaceLookup {
                owner: self.owner.utf8().into(),
                name: name.into(),
                desc: desc.into(),
            })
        } else {
            Err(ResolveError::NoSuchMethod {
                owner: self.owner.utf8().into(),
                name: name.into(),
                desc: desc.into(),
            })
        }
    }
}
