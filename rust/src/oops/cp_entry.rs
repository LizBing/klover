use std::{cell::OnceCell, marker::PhantomData, ptr::NonNull, sync::{OnceLock, mpsc::RecvError}};

use crate::{
    class_loader::{bootstrap_cld::BootstrapCLD, cld::ClassLoaderData, load_error::LoadResult, ms_api::MSRef}, class_parser::cp_info::ConstantPoolInfo, oops::{
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
    __: PhantomData<()>,
    
    pub holder: MSRef<NormalKlass>,
    pub field: MSRef<Field>,
}

#[derive(Debug, Clone)]
pub struct ResolvedMethodRef {
    __: PhantomData<()>,
    
    pub holder: MSRef<NormalKlass>,
    pub method: MSRef<Method>,
}

#[derive(Clone, Debug)]
pub struct ResolvedInterfaceMethodRef {
    __: PhantomData<()>,
    
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
) -> (SymbolHandle, SymbolHandle) {
    match cp[idx].get() {
        Some(x) => match x {
            CPEntry::NameAndType { name, desc } => (name.clone(), desc.clone()),
            _ => unreachable!(),
        },

        None => match &parsed_cp[idx] {
            ConstantPoolInfo::NameAndTypeInfo {
                name_index,
                desc_index,
            } => {
                let name = resolve_symbol(*name_index as usize, cp, parsed_cp);
                let desc = resolve_symbol(*desc_index as usize, cp, parsed_cp);

                cp[idx]
                    .set(CPEntry::NameAndType {
                        name: name.clone(),
                        desc: desc.clone(),
                    })
                    .unwrap();

                (name, desc)
            }

            _ => unreachable!(),
        },
    }
}

impl<R> CPRefEntry<R> {
    fn build(
        info: &ConstantPoolInfo,
        cp: &[OnceCell<CPEntry>],
        parsed_cp: &[ConstantPoolInfo],
    ) -> Self {
        match info {
            ConstantPoolInfo::FieldrefInfo {
                class_index,
                name_and_type_index,
            } => {
                let class = resolve_class_entry(*class_index as usize, cp, parsed_cp);
                let (name, desc) =
                    resolve_name_and_type(*name_and_type_index as usize, cp, parsed_cp);

                let symbolic = SymbolicMemberRef { class, name, desc };

                Self {
                    symbolic,
                    resolved: OnceLock::new(),
                }
            }

            ConstantPoolInfo::MethodrefInfo {
                class_index,
                name_and_type_index,
            } => {
                let class = resolve_class_entry(*class_index as usize, cp, parsed_cp);
                let (name, desc) =
                    resolve_name_and_type(*name_and_type_index as usize, cp, parsed_cp);

                let symbolic = SymbolicMemberRef { class, name, desc };

                Self {
                    symbolic,
                    resolved: OnceLock::new(),
                }
            }

            ConstantPoolInfo::InterfaceMethodrefInfo {
                class_index,
                name_and_type_index,
            } => {
                let class = resolve_class_entry(*class_index as usize, cp, parsed_cp);
                let (name, desc) =
                    resolve_name_and_type(*name_and_type_index as usize, cp, parsed_cp);

                let symbolic = SymbolicMemberRef { class, name, desc };

                Self {
                    symbolic,
                    resolved: OnceLock::new(),
                }
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
        let target = self
            .symbolic
            .class
            .resolve(referrer.cld())
            .map_err(|e| ResolveError::Load(e))?;
        
        let target = target.as_normal_ref().unwrap();
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
                __: PhantomData,

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
        let target = self
            .symbolic
            .class
            .resolve(referrer.cld())
            .map_err(|e| ResolveError::Load(e))?;

        let mut current = target.as_normal_ref().unwrap();

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
                __: PhantomData,

                holder: current,
                method,
            });
        }

        loop {
            if let Some(method) =
                current.find_declared_method_symbol(&self.symbolic.name, &self.symbolic.desc)
            {
                return Ok(ResolvedMethodRef {
                    __: PhantomData,
                    
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
    resolved: OnceLock<LoadResult<MSRef<Klass>>>,
}

impl ClassCPEntry {
    pub(super) fn set(&self, klass: MSRef<Klass>) {
        self.resolved.set(Ok(klass)).unwrap()
    }

    pub fn resolve(&self, cld: Option<&ClassLoaderData>) -> LoadResult<MSRef<Klass>> {
        self.resolved.get_or_init(|| self.resolve_slowpath(cld)).clone()
    }

    pub fn resolve_slowpath(&self, cld: Option<&ClassLoaderData>) -> LoadResult<MSRef<Klass>> {
        let klass = match cld {
            Some(x) => x.load_class(self.name.utf8()),
            None => BootstrapCLD::find_class(self.name.utf8()),
        }?;

        Ok(klass)
    }
}

#[derive(Debug)]
pub struct StringCPEntry {
    raw: SymbolHandle,
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
) -> SymbolHandle {
    match cp[idx].get() {
        Some(CPEntry::Class(entry)) => entry.name.clone(),

        None => match &parsed_cp[idx] {
            ConstantPoolInfo::ClassInfo { name_index } => {
                let name = resolve_symbol(*name_index as usize, cp, parsed_cp);

                cp[idx]
                    .set(CPEntry::Class(ClassCPEntry {
                        name: name.clone(),
                        resolved: OnceLock::new(),
                    }))
                    .unwrap();

                name
            }

            _ => unreachable!(),
        },

        _ => unreachable!()
    }
}

fn resolve_class_entry(
    index: usize,
    cp: &[OnceCell<CPEntry>],
    parsed_cp: &[ConstantPoolInfo],
) -> MSRef<ClassCPEntry> {
    resolve_class_symbol(index, cp, parsed_cp);

    match cp[index].get() {
        Some(CPEntry::Class(entry)) => unsafe { MSRef::from_raw(NonNull::from(entry)) },
        _ => unreachable!(),
    }
}

fn resolve_symbol(
    idx: usize,
    cp: &[OnceCell<CPEntry>],
    parsed_cp: &[ConstantPoolInfo],
) -> SymbolHandle {
    match cp[idx].get() {
        Some(x) => match x {
            CPEntry::Utf8(handle) => handle.clone(),
            _ => unreachable!(),
        },

        None => match &parsed_cp[idx] {
            ConstantPoolInfo::Utf8Info { utf8 } => {
                let handle = SymbolTable::intern(utf8.as_str());
                cp[idx].set(CPEntry::Utf8(handle.clone())).unwrap();

                handle
            }

            _ => unreachable!(),
        },
    }
}

impl CPEntry {
    pub fn from(
        idx: usize,
        cp: &[OnceCell<Self>],
        parsed_cp: &[ConstantPoolInfo],
    ) {
        let info = &parsed_cp[idx];

        let res = match info {
            ConstantPoolInfo::ClassInfo { name_index } => {
                let name = resolve_symbol(*name_index as usize, cp, parsed_cp);
                Self::Class(ClassCPEntry {
                    name,
                    resolved: OnceLock::new(),
                })
            }

            ConstantPoolInfo::FieldrefInfo { .. } => {
                let entry = CPRefEntry::build(info, cp, parsed_cp);
                Self::FieldRef(entry)
            }

            ConstantPoolInfo::MethodrefInfo { .. } => {
                let entry = CPRefEntry::build(info, cp, parsed_cp);
                Self::MethodRef(entry)
            }

            ConstantPoolInfo::InterfaceMethodrefInfo { .. } => {
                let entry = CPRefEntry::build(info, cp, parsed_cp);
                Self::InterfaceMethodRef(entry)
            }

            ConstantPoolInfo::StringInfo { string_index } => Self::StringConstant(StringCPEntry {
                raw: resolve_symbol(*string_index as usize, cp, parsed_cp),
            }),

            ConstantPoolInfo::IntegerInfo { value } => Self::Integer(*value),

            ConstantPoolInfo::FloatInfo { value } => Self::Float(*value),

            ConstantPoolInfo::LongInfo { value } => Self::Long(*value),

            ConstantPoolInfo::DoubleInfo { value } => Self::Double(*value),

            ConstantPoolInfo::NameAndTypeInfo { .. } => {
                let (name, desc) = resolve_name_and_type(idx, cp, parsed_cp);
                Self::NameAndType { name, desc }
            }

            ConstantPoolInfo::Utf8Info { .. } => {
                let handle = resolve_symbol(idx, cp, parsed_cp);
                Self::Utf8(handle)
            }

            ConstantPoolInfo::Unusable => return,
        };

        cp[idx].get_or_init(|| res);
    }
}

pub fn get_utf8(cp: &[OnceCell<CPEntry>], idx: usize) -> SymbolHandle {
    match cp[idx].get() {
        Some(CPEntry::Utf8(handle)) => handle.clone(),
        _ => unreachable!(),
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
