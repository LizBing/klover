use std::{cell::OnceCell, mem, sync::OnceLock};

use crate::{
    class_loader::ms_api::MSRef, class_parser::cp_info::ConstantPoolInfo, gc_bindings::oop_handle::{KLASS_OOP_STORAGE_ID, OOPHandle}, oops::{
        attr::BootstrapMethod, desc::MethodDesc, field::Field, klass::Klass, method::Method, normal_klass::NormalKlass, resolve_error::{ResolveError, ResolveResult}, symbol_table::{SymbolHandle, SymbolTable}
    }
};

#[derive(Debug)]
pub enum ResolvedRef {
    Field(MSRef<NormalKlass>, MSRef<Field>),
    Method(MSRef<NormalKlass>, MSRef<Method>)
}

#[derive(Debug)]
pub struct CPRefEntry {
    pub class_name: SymbolHandle,
    pub name: SymbolHandle,
    pub desc: SymbolHandle,

    pub resolved: OnceLock<ResolvedRef>,
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

#[derive(Debug)]
pub enum MethodHandleEntry {
    RefGetField(MSRef<CPRefEntry>),
    RefGetStatic(MSRef<CPRefEntry>),
    RefPutField(MSRef<CPRefEntry>),
    RefPutStatic(MSRef<CPRefEntry>),

    RefInvokeVirtual(MSRef<CPRefEntry>),
    RefInvokeStatic(MSRef<CPRefEntry>),
    RefInvokeSpecial(MSRef<CPRefEntry>),
    RefNewInvokeSpecial(MSRef<CPRefEntry>),
    RefInvokeInterface(MSRef<CPRefEntry>),
}

fn resolve_method_handle_entry(
    idx: usize,
    cp: &[OnceCell<CPEntry>],
    parsed_cp: &[ConstantPoolInfo],
) -> ResolveResult<MSRef<CPRefEntry>> {
    if let None = cp[idx].get() {
        let info = &parsed_cp[idx];
        let entry = CPRefEntry::build(info, cp, parsed_cp)?;

        let res = match info {
            ConstantPoolInfo::FieldrefInfo { .. } => CPEntry::FieldRef(entry),
            
            ConstantPoolInfo::MethodrefInfo { .. } => CPEntry::MethodRef(entry),
            
            ConstantPoolInfo::InterfaceMethodrefInfo { .. } => CPEntry::InterfaceMethodRef(entry),
            _ => return Err(ResolveError::MismatchCPType),
        };

        cp[idx].set(res).unwrap();
    }
    
    let entry = match cp[idx].get().unwrap() {
        CPEntry::FieldRef(x) => x,
        CPEntry::MethodRef(x) => x,
        CPEntry::InterfaceMethodRef(x) => x,

        _ => return Err(ResolveError::MismatchCPType),
    };

    Ok(entry.into())
}

impl MethodHandleEntry {
    fn from(
        ref_kind: u8,
        ref_index: usize,
        cp: &[OnceCell<CPEntry>],
        parsed_cp: &[ConstantPoolInfo],
    ) -> ResolveResult<Self> {
        match ref_kind {
            1 => {
                let entry = resolve_method_handle_entry(ref_index, cp, parsed_cp)?;
                Ok(Self::RefGetField(entry))
            }
            2 => {
                let entry = resolve_method_handle_entry(ref_index, cp, parsed_cp)?;
                Ok(Self::RefGetStatic(entry))
            }
            3 => {
                let entry = resolve_method_handle_entry(ref_index, cp, parsed_cp)?;
                Ok(Self::RefPutField(entry))
            }
            4 => {
                let entry = resolve_method_handle_entry(ref_index, cp, parsed_cp)?;
                Ok(Self::RefPutStatic(entry))
            }
            5 => {
                let entry = resolve_method_handle_entry(ref_index, cp, parsed_cp)?;
                Ok(Self::RefInvokeVirtual(entry))
            }
            6 => {
                let entry = resolve_method_handle_entry(ref_index, cp, parsed_cp)?;
                Ok(Self::RefInvokeStatic(entry))
            }
            7 => {
                let entry = resolve_method_handle_entry(ref_index, cp, parsed_cp)?;
                Ok(Self::RefInvokeSpecial(entry))
            }
            8 => {
                let entry = resolve_method_handle_entry(ref_index, cp, parsed_cp)?;
                Ok(Self::RefNewInvokeSpecial(entry))
            }
            9 => {
                let entry = resolve_method_handle_entry(ref_index, cp, parsed_cp)?;
                Ok(Self::RefInvokeInterface(entry))
            }

            _ => Err(ResolveError::UnknownRefKind { kind: ref_kind }),
        }
    }
}

#[derive(Debug)]
pub struct ClassCPEntry {
    pub name: SymbolHandle,
    pub resolved: OnceLock<MSRef<Klass>>,
}

#[derive(Debug)]
pub struct StringCPEntry {
    pub raw: SymbolHandle,
    pub resolved: OOPHandle,
}

#[derive(Debug)]
pub struct DynamicEntry {
    pub bs_method_attr_index: usize,
    pub bs_method: OnceCell<MSRef<BootstrapMethod>>,
    
    pub name: SymbolHandle,
    pub desc: MethodDesc,
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

    MethodHandle(MethodHandleEntry),

    MethodType(MethodDesc),

    // Ignore for now.
    Dynamic(DynamicEntry),

    // Ignore for now.
    InvokeDynamic {},

    Module {
        name: SymbolHandle,
    },

    Package {
        name: SymbolHandle,
    },
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

            ConstantPoolInfo::MethodHandleInfo {
                ref_kind,
                ref_index,
            } => {
                let entry = MethodHandleEntry::from(*ref_kind, *ref_index as usize, cp, parsed_cp)?;
                Self::MethodHandle(entry)
            }

            ConstantPoolInfo::MethodTypeInfo { desc_index } => {
                let raw = resolve_symbol(*desc_index as usize, cp, parsed_cp)?;
                Self::MethodType(MethodDesc::from(raw.utf8())?)
            }

            ConstantPoolInfo::DynamicInfo { bs_method_attr_index, name_and_type_index } => {
                let (name, raw_desc) = resolve_name_and_type(*name_and_type_index as usize, cp, parsed_cp)?;
                Self::Dynamic(DynamicEntry {
                    bs_method_attr_index: *bs_method_attr_index as usize,
                    bs_method: OnceCell::new(),
                    name,
                    desc: MethodDesc::from(raw_desc.utf8())?
                })
            }

            ConstantPoolInfo::InvokeDynamicInfo { .. } => Self::InvokeDynamic {},

            ConstantPoolInfo::ModuleInfo { .. } => {
                let name = resolve_symbol(idx, cp, parsed_cp)?;
                Self::Module { name }
            }

            ConstantPoolInfo::PackageInfo { .. } => {
                let name = resolve_symbol(idx, cp, parsed_cp)?;
                Self::Package { name }
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
    MethodHandle(MSRef<MethodHandleEntry>),
    MethodType(MethodDesc),
    Dynamic(MSRef<DynamicEntry>)
}
