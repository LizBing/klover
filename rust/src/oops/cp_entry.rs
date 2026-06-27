use std::{ptr::{NonNull, null_mut}, sync::atomic::AtomicPtr};
use once_cell::unsync::OnceCell;

use crate::{class_loader::ms_box::MSBox, class_parser::cp_info::ConstantPoolInfo, oops::{desc, field::Field, klass::Klass, method::Method, oop_handle::{KLASS_OOP_STORAGE_ID, OOPHandle}, resolve_error::{ResolveError, ResolveResult}, symbol_table::{SymbolHandle, SymbolTable}}};

pub struct CPRefEntry<T> {
    class_name: SymbolHandle,
    name: SymbolHandle,
    desc: SymbolHandle,

    resolved: AtomicPtr<MSBox<T>>
}

fn resolve_name_and_type(idx: usize, cp: &mut [Option<CPEntry>], parsed_cp: &[ConstantPoolInfo]) -> ResolveResult<(SymbolHandle, SymbolHandle)> {
    match &cp[idx] {
        Some(x) => match x {
            CPEntry::NameAndType { name, desc } => {
                Ok((name.clone(), desc.clone()))
            },

            _ => Err(ResolveError::MismatchCPType)
        },

        None => match &parsed_cp[idx] {
            ConstantPoolInfo::NameAndTypeInfo { name_index, desc_index } => {
                let name = resolve_symbol(*name_index as usize, cp, parsed_cp)?;
                let desc = resolve_symbol(*desc_index as usize, cp, parsed_cp)?;

                cp[idx] = Some(CPEntry::NameAndType { name: name.clone(), desc: desc.clone() });

                Ok((name, desc))
            }

            _ => Err(ResolveError::MismatchCPType)
        }
    }
}

impl<T> CPRefEntry<T> {
    fn from(info: &ConstantPoolInfo, cp: &mut [Option<CPEntry>], parsed_cp: &[ConstantPoolInfo]) -> ResolveResult<Self> {
        match info {
            ConstantPoolInfo::FieldrefInfo { class_index, name_and_type_index } => {
                let class_name = resolve_symbol(*class_index as usize, cp, parsed_cp)?;
                let (name, desc) = resolve_name_and_type(*name_and_type_index as usize, cp, parsed_cp)?;

                Ok(Self {
                    class_name,
                    name,
                    desc,
                    resolved: AtomicPtr::new(null_mut())
                })
            }
            
            ConstantPoolInfo::MethodrefInfo { class_index, name_and_type_index } => {
                let class_name = resolve_symbol(*class_index as usize, cp, parsed_cp)?;
                let (name, desc) = resolve_name_and_type(*name_and_type_index as usize, cp, parsed_cp)?;

                Ok(Self {
                    class_name,
                    name,
                    desc,
                    resolved: AtomicPtr::new(null_mut())
                })
            }

            ConstantPoolInfo::InterfaceMethodrefInfo { class_index, name_and_type_index } => {
                let class_name = resolve_symbol(*class_index as usize, cp, parsed_cp)?;
                let (name, desc) = resolve_name_and_type(*name_and_type_index as usize, cp, parsed_cp)?;

                Ok(Self {
                    class_name,
                    name,
                    desc,
                    resolved: AtomicPtr::new(null_mut())
                })
            }

            _ => unreachable!()
        }
    }
}

pub enum MethodHandleEntry {
    RefGetField(NonNull<CPRefEntry<Field>>),
    RefGetStatic(NonNull<CPRefEntry<Field>>),
    RefPutField(NonNull<CPRefEntry<Field>>),
    RefPutStatic(NonNull<CPRefEntry<Field>>),

    RefInvokeVirtual(NonNull<CPRefEntry<Method>>),
    RefInvokeStatic(NonNull<CPRefEntry<Method>>),
    RefInvokeSpecial(NonNull<CPRefEntry<Method>>),
    RefNewInvokeSpecial(NonNull<CPRefEntry<Method>>),
    RefInvokeInterface(NonNull<CPRefEntry<Method>>)
}

impl MethodHandleEntry {
    fn from(ref_kind: u8, ref_index: usize, cp: &mut [Option<CPEntry>], parsed_cp: &[ConstantPoolInfo]) -> ResolveResult<Self> {
        unimplemented!()
    }
}

pub enum CPEntry {
    Class {
        name: SymbolHandle,
        
        resolved: AtomicPtr<MSBox<Klass>>
    },

    FieldRef { entry: CPRefEntry<Field> },

    MethodRef { entry: CPRefEntry<Method> },

    InterfaceMethodRef { entry: CPRefEntry<Method> },

    StringObj {
        raw: SymbolHandle,

        resolved: OOPHandle
    },
    
    Integer {
        value: i32,
    },

    Float {
        value: f32,
    },

    Long {
        value: i64,
    },

    Double {
        value: f64,
    },

    NameAndType {
        name: SymbolHandle,
        desc: SymbolHandle
    },

    Utf8 {
        handle: SymbolHandle
    },

    MethodHandle { entry: MethodHandleEntry },

    MethodType {
        desc: SymbolHandle,
    },

    // Ignore for now.
    Dynamic { },

    // Ignore for now.
    InvokeDynamic { },

    Module {
        name: SymbolHandle
    },

    Package {
        name: SymbolHandle
    }
}

fn resolve_symbol(idx: usize, cp: &mut [Option<CPEntry>], parsed_cp: &[ConstantPoolInfo]) -> ResolveResult<SymbolHandle> {
    match &cp[idx] {
        Some(x) => match x {
            CPEntry::Utf8 { handle } => Ok(handle.clone()),

            _ => Err(ResolveError::MismatchCPType)
        },

        None => match &parsed_cp[idx] {
            ConstantPoolInfo::Utf8Info { utf8 } => {
                let handle = SymbolTable::intern(utf8.clone());
                cp[idx] = Some(CPEntry::Utf8 { handle: handle.clone() });

                Ok(handle)
            }

            _ => Err(ResolveError::MismatchCPType)
        }
    }
}

impl CPEntry {
    pub fn from(idx: usize, cp: &mut [Option<Self>], parsed_cp: &[ConstantPoolInfo]) -> ResolveResult<Option<Self>> {
        let info = &parsed_cp[idx];

        match info {
            ConstantPoolInfo::ClassInfo { .. } => {
                let name = resolve_symbol(idx, cp, parsed_cp)?;
                Ok(Some(Self::Class {
                    name,
                    resolved: AtomicPtr::new(null_mut())
                }))
            }

            ConstantPoolInfo::FieldrefInfo { .. } => {
                match cp.get(idx) {
                    Some(_) => Ok(None),
                    None => {
                        let entry = CPRefEntry::from(info, cp, parsed_cp)?;
                        Ok(Some(Self::FieldRef { entry }))
                    }
                }
            }

            ConstantPoolInfo::MethodrefInfo { .. } => {
                match cp.get(idx) {
                    Some(_) => Ok(None),
                    None => {
                        let entry = CPRefEntry::from(info, cp, parsed_cp)?;
                        Ok(Some(Self::MethodRef { entry }))
                    }
                }
            }

            ConstantPoolInfo::InterfaceMethodrefInfo { .. } => {
                match cp.get(idx) {
                    Some(_) => Ok(None),
                    None => {
                        let entry = CPRefEntry::from(info, cp, parsed_cp)?;
                        Ok(Some(Self::InterfaceMethodRef { entry }))
                    }
                }
            }

            ConstantPoolInfo::StringInfo { string_index } => {
                Ok(Some(Self::StringObj {
                    raw: resolve_symbol(*string_index as usize, cp, parsed_cp)?,
                    resolved: OOPHandle::new(KLASS_OOP_STORAGE_ID)
                }))
            }

            ConstantPoolInfo::IntegerInfo { value } => {
                Ok(Some(Self::Integer { value: *value }))
            }
            
            ConstantPoolInfo::FloatInfo { value } => {
                Ok(Some(Self::Float { value: *value }))
            }
            
            ConstantPoolInfo::LongInfo { value } => {
                Ok(Some(Self::Long { value: *value }))
            }
            
            ConstantPoolInfo::DoubleInfo { value } => {
                Ok(Some(Self::Double { value: *value }))
            }

            ConstantPoolInfo::NameAndTypeInfo { .. } => {
                match cp.get(idx) {
                    Some(_) => Ok(None),
                    None => {
                        let (name, desc) = resolve_name_and_type(idx, cp, parsed_cp)?;
                        
                        Ok(Some(Self::NameAndType {
                            name,
                            desc
                        }))
                    }
                }
            }

            ConstantPoolInfo::Utf8Info { .. } => {
                match cp.get(idx) {
                    Some(_) => Ok(None),
                    None => {
                        let handle = resolve_symbol(idx, cp, parsed_cp)?;

                        Ok(Some(Self::Utf8 { handle }))
                    }
                }
            }

            ConstantPoolInfo::MethodHandleInfo { ref_kind, ref_index } => {
                let entry = MethodHandleEntry::from(*ref_kind, *ref_index as usize, cp, parsed_cp)?;
                Ok(Some(Self::MethodHandle { entry }))
            }

            ConstantPoolInfo::MethodTypeInfo { desc_index } => {
                let desc = resolve_symbol(*desc_index as usize, cp, parsed_cp)?;
                Ok(Some(Self::MethodType { desc }))
            }

            ConstantPoolInfo::DynamicInfo { .. } => {
                Ok(Some(Self::Dynamic {  }))
            }

            ConstantPoolInfo::InvokeDynamicInfo { .. } => {
                Ok(Some(Self::InvokeDynamic {  }))
            }

            ConstantPoolInfo::ModuleInfo { .. } => {
                let name = resolve_symbol(idx, cp, parsed_cp)?;
                Ok(Some(Self::Module { name }))
            }

            ConstantPoolInfo::PackageInfo { .. } => {
                let name = resolve_symbol(idx, cp, parsed_cp)?;
                Ok(Some(Self::Package { name }))
            }

            ConstantPoolInfo::Unusable => unreachable!()
        }
    }
}
