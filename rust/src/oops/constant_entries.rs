use std::sync::OnceLock;

use crate::{class_loader::oop_storage::{OOPHandle, OOPStorage}, gc_bindings::oop_hierarchy::NObjPtr, oops::{jvalue::{JDouble, JFloat, JInt, JLong}, klass::Klass, oops_errors::ResolveResult, symbol_table::{SymbolHandle, SymbolTable}}};

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

#[derive(Debug, Clone)]
pub enum LdC2Loadable {
    Long(JLong),
    Double(JDouble),
}
