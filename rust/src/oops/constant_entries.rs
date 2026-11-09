use std::sync::OnceLock;

use crate::{
    class_parser::{FieldMemRef, MethodMemRef, MethodMemRefKind},
    oops::{
        fields::Field,
        jvalue::{JDouble, JFloat, JInt, JLong},
        klass::Klass,
        method::Method,
        normal_klass::NormalKlass,
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

#[derive(Debug, Clone, Copy)]
pub struct ResolvedMethod<'meta> {
    pub declaring_klass: &'meta NormalKlass<'meta>,
    pub method: &'meta Method<'meta>,
}

#[derive(Debug)]
pub struct MethodRef<'metadata> {
    is_interface: bool,
    owner: SymbolHandle,
    name: SymbolHandle,
    desc: SymbolHandle,
    resolved: OnceLock<ResolvedMethod<'metadata>>,
}

impl MethodRef<'_> {
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
    /// Only accessible successful lookups are cached. Access is checked for each
    /// caller, including cache hits. Loading constraints are not yet implemented.
    pub fn resolve(
        &self,
        caller: &NormalKlass<'metadata>,
    ) -> ResolveResult<ResolvedMethod<'metadata>> {
        let cld = caller.def_cld;
        let owner = cld.load_klass(&self.owner).map_err(ResolveError::Load)?;
        let Klass::Normal(owner) = owner else {
            unimplemented!("Method resolution for primitive and array classes")
        };

        if !owner.is_accessible_to(caller.name().utf8(), caller.def_cld) {
            return Err(ResolveError::IllegalClassAccess {
                from: caller.name().utf8().into(),
                target: self.owner.utf8().into(),
            });
        }

        if owner.is_interface() != self.is_interface {
            return Err(ResolveError::IncompatibleClassChange {
                owner: self.owner.utf8().into(),
                expected_interface: self.is_interface,
            });
        }

        if let Some(resolved) = self.resolved.get() {
            // A symbolic reference normally belongs to one caller's constant
            // pool. Do not reuse a target if a different loader uses this API.
            if std::ptr::eq(resolved.declaring_klass.def_cld, cld) {
                self.check_access(caller, owner, *resolved)?;
                return Ok(*resolved);
            }
        }

        let name = self.name.utf8();
        let desc = self.desc.utf8();
        let mut current = Some(owner);
        let mut needs_interface_lookup = self.is_interface;

        while let Some(klass) = current {
            if let Some(method) = klass.find_declared_method(name, desc) {
                let resolved = ResolvedMethod {
                    declaring_klass: klass,
                    method,
                };
                self.check_access(caller, owner, resolved)?;
                let _ = self.resolved.set(resolved);
                return Ok(resolved);
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

    fn check_access(
        &self,
        caller: &NormalKlass<'metadata>,
        owner: &NormalKlass<'metadata>,
        resolved: ResolvedMethod<'metadata>,
    ) -> ResolveResult<()> {
        let declaring = resolved.declaring_klass;
        let method = resolved.method;
        let allowed = if method.is_public() {
            true
        } else if method.is_private() {
            std::ptr::eq(caller, declaring)
        } else {
            declaring.same_runtime_package(caller)
                || (method.is_protected()
                    && caller.is_subclass_of(declaring)
                    && (method.is_static()
                        || caller.is_subclass_of(owner)
                        || owner.is_subclass_of(caller)))
        };
        if allowed {
            Ok(())
        } else {
            Err(ResolveError::IllegalMethodAccess {
                from: caller.name().utf8().into(),
                owner: declaring.name().utf8().into(),
                name: self.name.utf8().into(),
                desc: self.desc.utf8().into(),
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResolvedField<'meta> {
    pub declaring_klass: &'meta NormalKlass<'meta>,
    pub field: &'meta Field,
}

#[derive(Debug)]
pub struct FieldRef<'meta> {
    declaring_klass_name: SymbolHandle,
    field_name: SymbolHandle,
    desc: SymbolHandle,
    resolved: OnceLock<ResolvedField<'meta>>,
}

impl FieldRef<'_> {
    pub fn build(mref: &FieldMemRef) -> Self {
        let declaring_klass_name = SymbolTable::intern(&mref.owner.name);
        let field_name = SymbolTable::intern(&mref.name);
        let desc = SymbolTable::intern(mref.descriptor.raw());

        Self {
            declaring_klass_name,
            field_name,
            desc,
            resolved: OnceLock::new(),
        }
    }
}

impl<'meta> FieldRef<'meta> {
    pub fn resolve(&self, caller: &NormalKlass<'meta>) -> ResolveResult<ResolvedField<'meta>> {
        let owner = caller
            .def_cld
            .load_klass(&self.declaring_klass_name)
            .map_err(ResolveError::Load)?
            .as_normal()
            .expect("field owner must be a normal class");
        if !owner.is_accessible_to(caller.name().utf8(), caller.def_cld) {
            return Err(ResolveError::IllegalClassAccess {
                from: caller.name().utf8().into(),
                target: self.declaring_klass_name.utf8().into(),
            });
        }
        let resolved = match self
            .resolved
            .get()
            .filter(|resolved| std::ptr::eq(resolved.declaring_klass.def_cld, caller.def_cld))
        {
            Some(resolved) => *resolved,
            None => {
                let (declaring_klass, field) = owner
                    .resolve_field(self.field_name.utf8(), self.desc.utf8())
                    .ok_or_else(|| ResolveError::NoSuchField {
                        owner: self.declaring_klass_name.utf8().into(),
                        name: self.field_name.utf8().into(),
                        desc: self.desc.utf8().into(),
                    })?;
                ResolvedField {
                    declaring_klass,
                    field,
                }
            }
        };
        let declaring = resolved.declaring_klass;
        let field = resolved.field;
        let allowed = if field.is_public() {
            true
        } else if field.is_private() {
            std::ptr::eq(caller, declaring)
        } else {
            declaring.same_runtime_package(caller)
                || (field.is_protected()
                    && caller.is_subclass_of(declaring)
                    && (field.is_static()
                        || caller.is_subclass_of(owner)
                        || owner.is_subclass_of(caller)))
        };
        if !allowed {
            return Err(ResolveError::IllegalFieldAccess {
                from: caller.name().utf8().into(),
                owner: declaring.name().utf8().into(),
                name: self.field_name.utf8().into(),
                desc: self.desc.utf8().into(),
            });
        }
        let _ = self.resolved.set(resolved);
        Ok(resolved)
    }
}
