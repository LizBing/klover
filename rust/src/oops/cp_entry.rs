use std::{
    ptr::{NonNull, null_mut},
    sync::atomic::AtomicPtr,
};

use crate::{
    class_parser::cp_info::ConstantPoolInfo,
    oops::{
        field::Field,
        klass::Klass,
        method::Method,
        oop_handle::{KLASS_OOP_STORAGE_ID, OOPHandle},
        resolve_error::{ResolveError, ResolveResult},
        symbol_table::{SymbolHandle, SymbolTable},
    },
};

pub struct CPRefEntry<T> {
    class_name: SymbolHandle,
    name: SymbolHandle,
    desc: SymbolHandle,

    resolved: AtomicPtr<T>,
}

fn resolve_name_and_type(
    idx: usize,
    cp: &mut [Option<CPEntry>],
    parsed_cp: &[ConstantPoolInfo],
) -> ResolveResult<(SymbolHandle, SymbolHandle)> {
    match &cp[idx] {
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

                cp[idx] = Some(CPEntry::NameAndType {
                    name: name.clone(),
                    desc: desc.clone(),
                });

                Ok((name, desc))
            }

            _ => Err(ResolveError::MismatchCPType),
        },
    }
}

impl<T> CPRefEntry<T> {
    fn from(
        info: &ConstantPoolInfo,
        cp: &mut [Option<CPEntry>],
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
                    resolved: AtomicPtr::new(null_mut()),
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
                    resolved: AtomicPtr::new(null_mut()),
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
                    resolved: AtomicPtr::new(null_mut()),
                })
            }

            _ => unreachable!(),
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
    RefInvokeInterface(NonNull<CPRefEntry<Method>>),
}

fn resolve_method_handle_entry<T>(
    idx: usize,
    cp: &mut [Option<CPEntry>],
    parsed_cp: &[ConstantPoolInfo],
) -> ResolveResult<NonNull<CPRefEntry<T>>> {
    if cp[idx].is_none() {
        let info = &parsed_cp[idx];
        let entry = CPRefEntry::<T>::from(info, cp, parsed_cp)?;

        // CPRefEntry<T> has the same memory layout for any T because T only
        // appears in AtomicPtr<MSBox<T>>, and pointer sizes are identical.
        cp[idx] = Some(match info {
            ConstantPoolInfo::FieldrefInfo { .. } => CPEntry::FieldRef {
                entry: unsafe {
                    std::mem::transmute_copy::<CPRefEntry<T>, CPRefEntry<Field>>(&entry)
                },
            },
            ConstantPoolInfo::MethodrefInfo { .. } => CPEntry::MethodRef {
                entry: unsafe {
                    std::mem::transmute_copy::<CPRefEntry<T>, CPRefEntry<Method>>(&entry)
                },
            },
            ConstantPoolInfo::InterfaceMethodrefInfo { .. } => CPEntry::InterfaceMethodRef {
                entry: unsafe {
                    std::mem::transmute_copy::<CPRefEntry<T>, CPRefEntry<Method>>(&entry)
                },
            },
            _ => return Err(ResolveError::MismatchCPType),
        });
        // Prevent double-drop: ownership has been transferred via transmute_copy.
        std::mem::forget(entry);
    }

    match cp[idx].as_mut().unwrap() {
        CPEntry::FieldRef { entry } => Ok(NonNull::new(entry as *mut CPRefEntry<Field>)
            .unwrap()
            .cast()),
        CPEntry::MethodRef { entry } => Ok(NonNull::new(entry as *mut CPRefEntry<Method>)
            .unwrap()
            .cast()),
        CPEntry::InterfaceMethodRef { entry } => Ok(NonNull::new(entry as *mut CPRefEntry<Method>)
            .unwrap()
            .cast()),
        _ => Err(ResolveError::MismatchCPType),
    }
}

impl MethodHandleEntry {
    fn from(
        ref_kind: u8,
        ref_index: usize,
        cp: &mut [Option<CPEntry>],
        parsed_cp: &[ConstantPoolInfo],
    ) -> ResolveResult<Self> {
        match ref_kind {
            1 => {
                let entry = resolve_method_handle_entry::<Field>(ref_index, cp, parsed_cp)?;
                Ok(Self::RefGetField(entry))
            }
            2 => {
                let entry = resolve_method_handle_entry::<Field>(ref_index, cp, parsed_cp)?;
                Ok(Self::RefGetStatic(entry))
            }
            3 => {
                let entry = resolve_method_handle_entry::<Field>(ref_index, cp, parsed_cp)?;
                Ok(Self::RefPutField(entry))
            }
            4 => {
                let entry = resolve_method_handle_entry::<Field>(ref_index, cp, parsed_cp)?;
                Ok(Self::RefPutStatic(entry))
            }
            5 => {
                let entry = resolve_method_handle_entry::<Method>(ref_index, cp, parsed_cp)?;
                Ok(Self::RefInvokeVirtual(entry))
            }
            6 => {
                let entry = resolve_method_handle_entry::<Method>(ref_index, cp, parsed_cp)?;
                Ok(Self::RefInvokeStatic(entry))
            }
            7 => {
                let entry = resolve_method_handle_entry::<Method>(ref_index, cp, parsed_cp)?;
                Ok(Self::RefInvokeSpecial(entry))
            }
            8 => {
                let entry = resolve_method_handle_entry::<Method>(ref_index, cp, parsed_cp)?;
                Ok(Self::RefNewInvokeSpecial(entry))
            }
            9 => {
                let entry = resolve_method_handle_entry::<Method>(ref_index, cp, parsed_cp)?;
                Ok(Self::RefInvokeInterface(entry))
            }

            _ => Err(ResolveError::UnknownRefKind { kind: ref_kind }),
        }
    }
}

pub struct ClassCPEntry {
    pub name: SymbolHandle,
    pub resolved: AtomicPtr<Klass>,
}

pub struct StringEntry {
    pub raw: SymbolHandle,
    pub resolved: OOPHandle,
}

pub enum CPEntry {
    Class {
        entry: ClassCPEntry,
    },

    FieldRef {
        entry: CPRefEntry<Field>,
    },

    MethodRef {
        entry: CPRefEntry<Method>,
    },

    InterfaceMethodRef {
        entry: CPRefEntry<Method>,
    },

    StringConstant {
        entry: StringEntry,
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
        desc: SymbolHandle,
    },

    Utf8 {
        handle: SymbolHandle,
    },

    MethodHandle {
        entry: MethodHandleEntry,
    },

    MethodType {
        desc: SymbolHandle,
    },

    // Ignore for now.
    Dynamic {},

    // Ignore for now.
    InvokeDynamic {},

    Module {
        name: SymbolHandle,
    },

    Package {
        name: SymbolHandle,
    },
}

fn resolve_class_symbol(idx: usize, cp: &mut [Option<CPEntry>], parsed_cp: &[ConstantPoolInfo]) -> ResolveResult<SymbolHandle> {
    match &cp[idx] {
        Some(x) => match x {
            CPEntry::Class { entry } => Ok(entry.name.clone()),
            _ => Err(ResolveError::MismatchCPType),
        },

        None => match &parsed_cp[idx] {
            ConstantPoolInfo::ClassInfo { name_index } => {
                let name = resolve_symbol(*name_index as usize, cp, parsed_cp)?;

                cp[idx] = Some(CPEntry::Class { entry: ClassCPEntry { name: name.clone(), resolved: AtomicPtr::new(null_mut()) } });

                Ok(name)
            },

            _ => Err(ResolveError::MismatchCPType)
        }
    }
}

fn resolve_symbol(
    idx: usize,
    cp: &mut [Option<CPEntry>],
    parsed_cp: &[ConstantPoolInfo],
) -> ResolveResult<SymbolHandle> {
    match &cp[idx] {
        Some(x) => match x {
            CPEntry::Utf8 { handle } => Ok(handle.clone()),

            _ => Err(ResolveError::MismatchCPType),
        },

        None => match &parsed_cp[idx] {
            ConstantPoolInfo::Utf8Info { utf8 } => {
                let handle = SymbolTable::intern(utf8.clone());
                cp[idx] = Some(CPEntry::Utf8 {
                    handle: handle.clone(),
                });

                Ok(handle)
            }

            _ => Err(ResolveError::MismatchCPType),
        },
    }
}

impl CPEntry {
    pub fn from(
        idx: usize,
        cp: &mut [Option<Self>],
        parsed_cp: &[ConstantPoolInfo],
    ) -> ResolveResult<Option<Self>> {
        let info = &parsed_cp[idx];

        match info {
            ConstantPoolInfo::ClassInfo { name_index } => {
                let name = resolve_symbol(*name_index as usize, cp, parsed_cp)?;
                Ok(Some(Self::Class {
                    entry: ClassCPEntry {
                        name,
                        resolved: AtomicPtr::new(null_mut()),
                    },
                }))
            }

            ConstantPoolInfo::FieldrefInfo { .. } => {
                if cp[idx].is_some() {
                    Ok(None)
                } else {
                    let entry = CPRefEntry::from(info, cp, parsed_cp)?;
                    Ok(Some(Self::FieldRef { entry }))
                }
            }

            ConstantPoolInfo::MethodrefInfo { .. } => {
                if cp[idx].is_some() {
                    Ok(None)
                } else {
                    let entry = CPRefEntry::from(info, cp, parsed_cp)?;
                    Ok(Some(Self::MethodRef { entry }))
                }
            }

            ConstantPoolInfo::InterfaceMethodrefInfo { .. } => {
                if cp[idx].is_some() {
                    Ok(None)
                } else {
                    let entry = CPRefEntry::from(info, cp, parsed_cp)?;
                    Ok(Some(Self::InterfaceMethodRef { entry }))
                }
            }

            ConstantPoolInfo::StringInfo { string_index } => Ok(Some(Self::StringConstant {
                entry: StringEntry {
                    raw: resolve_symbol(*string_index as usize, cp, parsed_cp)?,
                    resolved: OOPHandle::new(KLASS_OOP_STORAGE_ID),
                },
            })),

            ConstantPoolInfo::IntegerInfo { value } => Ok(Some(Self::Integer { value: *value })),

            ConstantPoolInfo::FloatInfo { value } => Ok(Some(Self::Float { value: *value })),

            ConstantPoolInfo::LongInfo { value } => Ok(Some(Self::Long { value: *value })),

            ConstantPoolInfo::DoubleInfo { value } => Ok(Some(Self::Double { value: *value })),

            ConstantPoolInfo::NameAndTypeInfo { .. } => {
                if cp[idx].is_some() {
                    Ok(None)
                } else {
                    let (name, desc) = resolve_name_and_type(idx, cp, parsed_cp)?;
                    Ok(Some(Self::NameAndType { name, desc }))
                }
            }

            ConstantPoolInfo::Utf8Info { .. } => {
                if cp[idx].is_some() {
                    Ok(None)
                } else {
                    let handle = resolve_symbol(idx, cp, parsed_cp)?;
                    Ok(Some(Self::Utf8 { handle }))
                }
            }

            ConstantPoolInfo::MethodHandleInfo {
                ref_kind,
                ref_index,
            } => {
                let entry = MethodHandleEntry::from(*ref_kind, *ref_index as usize, cp, parsed_cp)?;
                Ok(Some(Self::MethodHandle { entry }))
            }

            ConstantPoolInfo::MethodTypeInfo { desc_index } => {
                let desc = resolve_symbol(*desc_index as usize, cp, parsed_cp)?;
                Ok(Some(Self::MethodType { desc }))
            }

            ConstantPoolInfo::DynamicInfo { .. } => Ok(Some(Self::Dynamic {})),

            ConstantPoolInfo::InvokeDynamicInfo { .. } => Ok(Some(Self::InvokeDynamic {})),

            ConstantPoolInfo::ModuleInfo { .. } => {
                let name = resolve_symbol(idx, cp, parsed_cp)?;
                Ok(Some(Self::Module { name }))
            }

            ConstantPoolInfo::PackageInfo { .. } => {
                let name = resolve_symbol(idx, cp, parsed_cp)?;
                Ok(Some(Self::Package { name }))
            }

            ConstantPoolInfo::Unusable => unreachable!(),
        }
    }
}

pub fn get_utf8(cp: &[Option<CPEntry>], idx: usize) -> ResolveResult<SymbolHandle> {
    match unsafe { cp[idx].as_ref().unwrap_unchecked() } {
        CPEntry::Utf8 { handle } => Ok(handle.clone()),
        _ => Err(ResolveError::MismatchCPType),
    }
}
