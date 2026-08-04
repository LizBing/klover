use std::{cell::OnceCell, ptr::NonNull, sync::OnceLock};

use crate::{
    class_loader::{bootstrap_cld::BootstrapCLD, cld::ClassLoaderData, ms_api::MSRef},
    class_parser::cp_info::ConstantPoolInfo,
    gc_bindings::oop_handle::{KLASS_OOP_STORAGE_ID, OOPHandle},
    oops::{
        desc::MethodDesc,
        field::Field,
        klass::Klass,
        method::Method,
        normal_klass::NormalKlass,
        oops_errors::{ResolveError, ResolveResult},
        symbol_table::{SymbolHandle, SymbolTable},
    },
};

#[derive(Debug)]
pub struct SymbolicMemberRef {
    class: MSRef<ClassCPEntry>,
    name: SymbolHandle,
    desc: SymbolHandle,
}

#[derive(Debug, Clone)]
pub struct ResolvedFieldRef {
    pub holder: MSRef<NormalKlass>,
    pub field: MSRef<Field>,
}

#[derive(Debug, Clone)]
pub struct ResolvedMethodRef {
    pub holder: MSRef<NormalKlass>,
    pub method: MSRef<Method>,
}

#[derive(Clone, Debug)]
pub struct ResolvedInterfaceMethodRef {
    pub holder: MSRef<NormalKlass>,
    pub method: MSRef<Method>,
}

#[derive(Debug)]
pub struct CPRefEntry<R> {
    symbolic: SymbolicMemberRef,
    resolved: OnceLock<ResolveResult<R>>,
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

                cp[idx]
                    .set(CPEntry::NameAndType {
                        name: name.clone(),
                        desc: desc.clone(),
                    })
                    .unwrap();

                Ok((name, desc))
            }

            _ => Err(ResolveError::MismatchCPType),
        },
    }
}

impl<R> CPRefEntry<R> {
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
                let class = resolve_class_entry(*class_index as usize, cp, parsed_cp)?;
                let (name, desc) =
                    resolve_name_and_type(*name_and_type_index as usize, cp, parsed_cp)?;

                let symbolic = SymbolicMemberRef { class, name, desc };

                Ok(Self {
                    symbolic,
                    resolved: OnceLock::new(),
                })
            }

            ConstantPoolInfo::MethodrefInfo {
                class_index,
                name_and_type_index,
            } => {
                let class = resolve_class_entry(*class_index as usize, cp, parsed_cp)?;
                let (name, desc) =
                    resolve_name_and_type(*name_and_type_index as usize, cp, parsed_cp)?;

                let symbolic = SymbolicMemberRef { class, name, desc };

                Ok(Self {
                    symbolic,
                    resolved: OnceLock::new(),
                })
            }

            ConstantPoolInfo::InterfaceMethodrefInfo {
                class_index,
                name_and_type_index,
            } => {
                let class = resolve_class_entry(*class_index as usize, cp, parsed_cp)?;
                let (name, desc) =
                    resolve_name_and_type(*name_and_type_index as usize, cp, parsed_cp)?;

                let symbolic = SymbolicMemberRef { class, name, desc };

                Ok(Self {
                    symbolic,
                    resolved: OnceLock::new(),
                })
            }

            _ => unreachable!(),
        }
    }
}

impl CPRefEntry<ResolvedFieldRef> {
    pub(super) fn resolve(&self, referrer: &NormalKlass) -> ResolveResult<ResolvedFieldRef> {
        self.resolved
            .get_or_init(|| self.resolve_slow_path(referrer))
            .clone()
    }

    fn resolve_slow_path(&self, referrer: &NormalKlass) -> ResolveResult<ResolvedFieldRef> {
        let target = self.symbolic.class.get(referrer.cld())?;
        let target = target.as_normal_ref().ok_or(ResolveError::NotANormal)?;
        let mut visited = Vec::new();

        Self::lookup_field(
            target,
            &self.symbolic.name,
            &self.symbolic.desc,
            &mut visited,
        )
        .ok_or(ResolveError::FieldNotFound)
    }

    /// JVMS 5.4.3.2 field lookup order: the current type, its direct
    /// superinterfaces recursively, and finally its superclass recursively.
    /// The returned holder is always the type that actually declares the field.
    fn lookup_field(
        current: MSRef<NormalKlass>,
        name: &SymbolHandle,
        desc: &SymbolHandle,
        visited: &mut Vec<MSRef<NormalKlass>>,
    ) -> Option<ResolvedFieldRef> {
        // 排除菱形继承
        if visited.iter().any(|seen| seen.equals(&current)) {
            return None;
        }
        visited.push(current.clone());

        if let Some(field) = current.find_declared_field_symbol(name, desc) {
            return Some(ResolvedFieldRef {
                holder: current,
                field,
            });
        }

        for interface in current.direct_interfaces() {
            if let Some(resolved) = Self::lookup_field(interface.clone(), name, desc, visited) {
                return Some(resolved);
            }
        }

        let super_klass = current.super_klass_ref()?;
        Self::lookup_field(super_klass, name, desc, visited)
    }
}

impl CPRefEntry<ResolvedMethodRef> {
    pub(super) fn resolve(&self, referrer: &NormalKlass) -> ResolveResult<ResolvedMethodRef> {
        self.resolved
            .get_or_init(|| self.resolve_slow_path(referrer))
            .clone()
    }

    fn resolve_slow_path(&self, referrer: &NormalKlass) -> ResolveResult<ResolvedMethodRef> {
        let target = self.symbolic.class.get(referrer.cld())?;

        let mut current = target.as_normal_ref().ok_or(ResolveError::NotANormal)?;

        if current.is_interface() {
            return Err(ResolveError::WrongRefType);
        }

        if self.symbolic.name.utf8() == "<clinit>" {
            return Err(ResolveError::IllegalMethodName("<clinit>".into()));
        }

        if self.symbolic.name.utf8() == "<init>" {
            let method = current
                .find_declared_method_symbol(&self.symbolic.name, &self.symbolic.desc)
                .ok_or(ResolveError::MethodNotFound)?;

            return Ok(ResolvedMethodRef {
                holder: current,
                method,
            });
        }

        loop {
            if let Some(method) =
                current.find_declared_method_symbol(&self.symbolic.name, &self.symbolic.desc)
            {
                return Ok(ResolvedMethodRef {
                    holder: current,
                    method,
                });
            }

            current = current
                .super_klass_ref()
                .ok_or(ResolveError::MethodNotFound)?;
        }
    }
}

impl CPRefEntry<ResolvedInterfaceMethodRef> {}

#[derive(Debug)]
pub struct ClassCPEntry {
    name: SymbolHandle,
    resolved: OnceLock<MSRef<Klass>>,
}

impl ClassCPEntry {
    pub fn set(&self, klass: MSRef<Klass>) {
        if let Err(candidate) = self.resolved.set(klass) {
            let existing = self
                .resolved
                .get()
                .expect("ClassCPEntry initialized concurrently but value is missing");

            assert!(
                existing.equals(&candidate),
                "ClassCPEntry resolved to different Klass instances"
            );
        }
    }

    pub fn get(&self, cld: Option<&ClassLoaderData>) -> ResolveResult<MSRef<Klass>> {
        if let Some(x) = self.resolved.get() {
            return Ok(x.clone());
        }

        let loaded = match cld {
            Some(x) => x.load_class(self.name.utf8()),
            None => BootstrapCLD::find_class(self.name.utf8()),
        }
        .map_err(|_| ResolveError::ClassNotFound)?;

        if self.resolved.set(loaded.clone()).is_ok() {
            return Ok(loaded);
        }

        Ok(self
            .resolved
            .get()
            .expect("resolved class missing after race")
            .clone())
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

    FieldRef(CPRefEntry<ResolvedFieldRef>),

    MethodRef(CPRefEntry<ResolvedMethodRef>),

    InterfaceMethodRef(CPRefEntry<ResolvedInterfaceMethodRef>),

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

                cp[idx]
                    .set(CPEntry::Class(ClassCPEntry {
                        name: name.clone(),
                        resolved: OnceLock::new(),
                    }))
                    .unwrap();

                Ok(name)
            }

            _ => Err(ResolveError::MismatchCPType),
        },
    }
}

fn resolve_class_entry(
    index: usize,
    cp: &[OnceCell<CPEntry>],
    parsed_cp: &[ConstantPoolInfo],
) -> ResolveResult<MSRef<ClassCPEntry>> {
    resolve_class_symbol(index, cp, parsed_cp)?;

    match cp[index].get() {
        Some(CPEntry::Class(entry)) => unsafe { Ok(MSRef::from_raw(NonNull::from(entry))) },
        _ => Err(ResolveError::MismatchCPType),
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

            ConstantPoolInfo::StringInfo { string_index } => Self::StringConstant(StringCPEntry {
                raw: resolve_symbol(*string_index as usize, cp, parsed_cp)?,
                resolved: OOPHandle::new(KLASS_OOP_STORAGE_ID),
            }),

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
