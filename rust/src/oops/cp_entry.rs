use std::{cell::OnceCell, sync::OnceLock};

use crate::{
    class_loader::{cld::ClassLoaderData, ms_api::MSRef}, class_parser::cp_info::ConstantPoolInfo, gc_bindings::oop_handle::{KLASS_OOP_STORAGE_ID, OOPHandle}, oops::{
        desc::MethodDesc, field::Field, klass::Klass, method::Method, normal_klass::NormalKlass, resolve_error::{ResolveError, ResolveResult}, symbol_table::{SymbolHandle, SymbolTable}
    }
};

#[derive(Debug)]
enum ResolvedRef {
    Field(MSRef<NormalKlass>, MSRef<Field>),
    Method(MSRef<NormalKlass>, MSRef<Method>)
}

#[derive(Debug)]
pub struct CPRefEntry {
    class_name: SymbolHandle,
    name: SymbolHandle,
    desc: SymbolHandle,

    resolved: OnceLock<ResolvedRef>,
}

fn resolve_name_and_type(
    idx: usize,
    cp: &[OnceCell<CPEntry>],
    parsed_cp: &[ConstantPoolInfo],
) -> ResolveResult<(SymbolHandle, SymbolHandle)> {
    match cp[idx].get() {
        Some(x) => match x {
            CPEntry::NameAndType { name, desc } => Ok((name.clone(), desc.clone())),

            _ => Err(ResolveError::MismatchCPType),
        },

        None => match &parsed_cp[idx] {
            ConstantPoolInfo::NameAndTypeInfo {
                name_index,
                desc_index,
            } => {
                let name = resolve_symbol(*name_index as usize, cp, parsed_cp)?;
                let desc = resolve_symbol(*desc_index as usize, cp, parsed_cp)?;

                cp[idx].set(CPEntry::NameAndType {
                    name: name.clone(),
                    desc: desc.clone(),
                }).unwrap();

                Ok((name, desc))
            }

            _ => Err(ResolveError::MismatchCPType),
        },
    }
}

impl CPRefEntry {
    fn build(
        info: &ConstantPoolInfo,
        cp: &[OnceCell<CPEntry>],
        parsed_cp: &[ConstantPoolInfo],
    ) -> ResolveResult<Self> {
        match info {
            ConstantPoolInfo::FieldrefInfo {
                class_index,
                name_and_type_index,
            } => {
                let class_name = resolve_class_symbol(*class_index as usize, cp, parsed_cp)?;
                let (name, desc) =
                    resolve_name_and_type(*name_and_type_index as usize, cp, parsed_cp)?;

                Ok(Self {
                    class_name,
                    name,
                    desc,
                    resolved: OnceLock::new(),
                })
            }

            ConstantPoolInfo::MethodrefInfo {
                class_index,
                name_and_type_index,
            } => {
                let class_name = resolve_class_symbol(*class_index as usize, cp, parsed_cp)?;
                let (name, desc) =
                    resolve_name_and_type(*name_and_type_index as usize, cp, parsed_cp)?;

                Ok(Self {
                    class_name,
                    name,
                    desc,
                    resolved: OnceLock::new(),
                })
            }

            ConstantPoolInfo::InterfaceMethodrefInfo {
                class_index,
                name_and_type_index,
            } => {
                let class_name = resolve_class_symbol(*class_index as usize, cp, parsed_cp)?;
                let (name, desc) =
                    resolve_name_and_type(*name_and_type_index as usize, cp, parsed_cp)?;

                Ok(Self {
                    class_name,
                    name,
                    desc,
                    resolved: OnceLock::new(),
                })
            }

            _ => unreachable!(),
        }
    }
}

impl CPRefEntry {
    pub fn get_fieldref(&self, cld: Option<&ClassLoaderData>) -> ResolveResult<(MSRef<NormalKlass>, MSRef<Field>)> {
        unimplemented!()
    }
    
    pub fn get_methodref(&self, cld: Option<&ClassLoaderData>) -> ResolveResult<(MSRef<NormalKlass>, MSRef<Method>)> {
        unimplemented!()
    }
    
    pub fn get_interface_methodref(&self, cld: Option<&ClassLoaderData>) -> ResolveResult<(MSRef<NormalKlass>, MSRef<Method>)> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct ClassCPEntry {
    name: SymbolHandle,
    resolved: OnceLock<MSRef<Klass>>,
}

impl ClassCPEntry {
    pub fn set(&self, klass: MSRef<Klass>) {
        self.resolved.set(klass).unwrap()
    }
    
    pub fn get(&self, cld: Option<&ClassLoaderData>) -> ResolveResult<MSRef<Klass>> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct StringCPEntry {
    raw: SymbolHandle,
    resolved: OOPHandle,
}

impl StringCPEntry {
    pub fn get(&self) -> &OOPHandle {
        unimplemented!()
    }
}

#[derive(Debug)]
pub enum CPEntry {
    Class(ClassCPEntry),

    FieldRef(CPRefEntry),

    MethodRef(CPRefEntry),

    InterfaceMethodRef(CPRefEntry),

    StringConstant(StringCPEntry),

    Integer(i32),

    Float(f32),

    Long(i64),

    Double(f64),

    NameAndType {
        name: SymbolHandle,
        desc: SymbolHandle,
    },

    Utf8(SymbolHandle),

    // Ignore for now.
    InvokeDynamic {},
}

fn resolve_class_symbol(
    idx: usize,
    cp: &[OnceCell<CPEntry>],
    parsed_cp: &[ConstantPoolInfo],
) -> ResolveResult<SymbolHandle> {
    match cp[idx].get() {
        Some(x) => match x {
            CPEntry::Class(entry) => Ok(entry.name.clone()),
            _ => Err(ResolveError::MismatchCPType),
        },

        None => match &parsed_cp[idx] {
            ConstantPoolInfo::ClassInfo { name_index } => {
                let name = resolve_symbol(*name_index as usize, cp, parsed_cp)?;

                cp[idx].set(CPEntry::Class(ClassCPEntry {
                    name: name.clone(),
                    resolved: OnceLock::new(),
                })).unwrap();

                Ok(name)
            }

            _ => Err(ResolveError::MismatchCPType),
        },
    }
}

fn resolve_symbol(
    idx: usize,
    cp: &[OnceCell<CPEntry>],
    parsed_cp: &[ConstantPoolInfo],
) -> ResolveResult<SymbolHandle> {
    match cp[idx].get() {
        Some(x) => match x {
            CPEntry::Utf8(handle) => Ok(handle.clone()),
            _ => Err(ResolveError::MismatchCPType),
        },

        None => match &parsed_cp[idx] {
            ConstantPoolInfo::Utf8Info { utf8 } => {
                let handle = SymbolTable::intern(utf8.as_str());
                cp[idx].set(CPEntry::Utf8(handle.clone())).unwrap();

                Ok(handle)
            }

            _ => Err(ResolveError::MismatchCPType),
        },
    }
}

impl CPEntry {
    pub fn from(
        idx: usize,
        cp: &[OnceCell<Self>],
        parsed_cp: &[ConstantPoolInfo],
    ) -> ResolveResult<()> {
        let info = &parsed_cp[idx];

        let res = match info {
            ConstantPoolInfo::ClassInfo { name_index } => {
                let name = resolve_symbol(*name_index as usize, cp, parsed_cp)?;
                Self::Class(ClassCPEntry {
                    name,
                    resolved: OnceLock::new(),
                })
            }

            ConstantPoolInfo::FieldrefInfo { .. } => {
                let entry = CPRefEntry::build(info, cp, parsed_cp)?;
                Self::FieldRef(entry)
            }

            ConstantPoolInfo::MethodrefInfo { .. } => {
                let entry = CPRefEntry::build(info, cp, parsed_cp)?;
                Self::MethodRef(entry)
            }

            ConstantPoolInfo::InterfaceMethodrefInfo { .. } => {
                let entry = CPRefEntry::build(info, cp, parsed_cp)?;
                Self::InterfaceMethodRef(entry)
            }

            ConstantPoolInfo::StringInfo { string_index } => {
                Self::StringConstant(StringCPEntry {
                    raw: resolve_symbol(*string_index as usize, cp, parsed_cp)?,
                    resolved: OOPHandle::new(KLASS_OOP_STORAGE_ID),
                })
            }

            ConstantPoolInfo::IntegerInfo { value } => Self::Integer(*value),

            ConstantPoolInfo::FloatInfo { value } => Self::Float(*value),

            ConstantPoolInfo::LongInfo { value } => Self::Long(*value),

            ConstantPoolInfo::DoubleInfo { value } => Self::Double(*value),

            ConstantPoolInfo::NameAndTypeInfo { .. } => {
                let (name, desc) = resolve_name_and_type(idx, cp, parsed_cp)?;
                Self::NameAndType { name, desc }
            }

            ConstantPoolInfo::Utf8Info { .. } => {
                let handle = resolve_symbol(idx, cp, parsed_cp)?;
                Self::Utf8(handle)
            }

            ConstantPoolInfo::Unusable => return Ok(()),
        };

        cp[idx].get_or_init(|| res);

        Ok(())
    }
}

pub fn get_utf8(cp: &[OnceCell<CPEntry>], idx: usize) -> ResolveResult<SymbolHandle> {
    match cp[idx].get() {
        Some(CPEntry::Utf8(handle)) => Ok(handle.clone()),
        _ => Err(ResolveError::MismatchCPType),
    }
}

#[derive(Debug)]
pub enum Loadable {
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class(MSRef<ClassCPEntry>),
    StringLoadable(MSRef<StringCPEntry>),
    MethodType(MethodDesc),
}
