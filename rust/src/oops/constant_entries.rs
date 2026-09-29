use std::sync::OnceLock;

use crate::{class_loader::oop_storage::{OOPHandle, OOPStorage}, gc_bindings::oop_hierarchy::NObjPtr, oops::{desc::MethodDesc, jvalue::{JDouble, JFloat, JInt, JLong}, klass::Klass, oops_errors::ResolveResult, symbol_table::{SymbolHandle, SymbolTable}}, runtime::ms_api::{MsAllocator, MsRef}};

#[derive(Debug)]
pub struct StringConstant {
    raw: SymbolHandle,
    obj: OOPHandle,
}

impl StringConstant {
    pub fn new(raw: &str, storage: &OOPStorage) -> Self {
        Self {
            raw: SymbolTable::intern(raw),
            obj: storage.create_strong(),
        }
    }

    pub fn resolve(&self) -> NObjPtr {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct ClassConstant {
    name: SymbolHandle,
    resolved: OnceLock<ResolveResult<MsRef<Klass>>>,
}

impl ClassConstant {
    pub fn new(name: &str) -> Self {
        Self {
            name: SymbolTable::intern(name),
            resolved: OnceLock::new(),
        }
    }
    
    pub fn resolve(&self) -> ResolveResult<MsRef<Klass>> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct MethodTypeConstant {
    desc: MethodDesc,
    obj: OOPHandle,
}

#[derive(Debug)]
pub struct MethodHandleConstant {
    obj: OOPHandle,
}

#[derive(Debug)]
pub enum LdCLoadable {
    Integer(JInt),
    Float(JFloat),
    String(StringConstant),
    Class(ClassConstant),
    MethodType(MethodTypeConstant),
    MethodHandle(MethodHandleConstant),
}

#[derive(Debug)]
pub enum LdC2Loadable {
    Long(JLong),
    Double(JDouble),
}
