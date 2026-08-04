use std::{
    cell::OnceCell,
    ptr::{NonNull, null},
};

use crate::{
    class_loader::{
        bootstrap_cld::BootstrapCLD,
        cld::ClassLoaderData,
        ms_api::{MSAllocator, MSBox, MSRef},
    },
    class_parser::{class_file::ClassFile, cp_info::ConstantPoolInfo, method_info::MethodInfo},
    engine::{exec_error::ExecResult, slot::Slot},
    gc_bindings::obj_layout::ObjLayout,
    oops::{
        acc_flags::AccFlags,
        cp_entry::{CPEntry, ClassCPEntry, ResolvedFieldRef, ResolvedMethodRef},
        field::Field,
        fields::Fields,
        klass::Klass,
        method::Method,
        oops_errors::{ClassInitError, ClassInitResult, ResolveError, ResolveResult},
        symbol_table::{SymbolHandle, SymbolTable},
    },
    runtime::java_thread::JavaThreadID,
};

#[derive(Debug)]
pub struct UnlinkedNormalKlass {
    acc_flags: AccFlags,

    this_klass: MSRef<ClassCPEntry>,
    pub super_klass: Option<MSRef<ClassCPEntry>>,

    constant_pool: MSBox<[OnceCell<CPEntry>]>,

    interfaces: Vec<MSRef<ClassCPEntry>>,

    fields: Fields,

    methods: MSBox<[Method]>,
}

fn build_cp<'a>(
    parsed_cp: &[ConstantPoolInfo],
    msa: &MSAllocator,
) -> ResolveResult<MSBox<[OnceCell<CPEntry>]>> {
    let cp_len = parsed_cp.len();
    let uninit = msa.calloc(cp_len);

    for i in 0..cp_len {
        uninit[i].write(OnceCell::new());
    }

    let cp = unsafe { MSBox::from_raw(uninit.assume_init_mut()) };

    for i in 1..cp_len {
        CPEntry::from(i, &cp, parsed_cp)?;
    }

    Ok(cp)
}

pub fn cp_slice_get(cp_slice: &[OnceCell<CPEntry>], idx: usize) -> Option<&CPEntry> {
    cp_slice[idx].get()
}

fn build_interfaces(
    parsed_ifaces: &[u16],
    cp_slice: &[OnceCell<CPEntry>],
) -> ResolveResult<Vec<MSRef<ClassCPEntry>>> {
    let mut ifaces = Vec::with_capacity(parsed_ifaces.len());

    for idx in parsed_ifaces {
        match cp_slice_get(cp_slice, *idx as usize) {
            Some(CPEntry::Class(entry)) => unsafe {
                ifaces.push(MSRef::from_raw(entry.into()));
            },
            _ => return Err(ResolveError::MismatchCPType),
        };
    }

    Ok(ifaces)
}

fn link_interfaces(
    entries: &[MSRef<ClassCPEntry>],
    cld: Option<&ClassLoaderData>,
    msa: &MSAllocator,
) -> ResolveResult<MSBox<[MSRef<NormalKlass>]>> {
    let uninit = msa.calloc(entries.len());

    for (i, entry) in entries.iter().enumerate() {
        let klass = entry.get(cld)?;
        let interface = klass.as_normal_ref().ok_or(ResolveError::NotANormal)?;
        if !interface.is_interface() {
            return Err(ResolveError::WrongRefType);
        }
        uninit[i].write(interface);
    }

    unsafe { Ok(MSBox::from_raw(uninit.assume_init_mut())) }
}

fn build_methods(
    parsed_methods: &[MethodInfo],
    cp_slice: &[OnceCell<CPEntry>],
    msa: &MSAllocator,
) -> ResolveResult<MSBox<[Method]>> {
    let methods_len = parsed_methods.len();
    let uninit = msa.calloc(methods_len);

    for (i, info) in parsed_methods.iter().enumerate() {
        uninit[i].write(Method::from(info, cp_slice, msa)?);
    }

    unsafe { Ok(MSBox::from_raw(uninit.assume_init_mut())) }
}

impl UnlinkedNormalKlass {
    pub fn build(cf: ClassFile, cld: Option<&ClassLoaderData>) -> ResolveResult<Self> {
        let msa = match cld {
            Some(x) => &x.ms_allocator,
            None => BootstrapCLD::bs_msa(),
        };

        let acc_flags = AccFlags::from_bits_truncate(cf.acc_flags);

        let cp = build_cp(&cf.constant_pool, msa)?;

        let this_entry: MSRef<ClassCPEntry> = match cp_slice_get(&cp, cf.this_class as usize) {
            Some(CPEntry::Class(entry)) => unsafe { MSRef::from_raw(entry.into()) },
            _ => return Err(ResolveError::MismatchCPType),
        };

        let super_entry = if cf.super_index == 0 {
            None
        } else {
            Some(match cp_slice_get(&cp, cf.super_index as usize) {
                Some(CPEntry::Class(entry)) => unsafe { MSRef::from_raw(entry.into()) },
                _ => return Err(ResolveError::MismatchCPType),
            })
        };

        let interfaces = build_interfaces(&cf.interfaces, &cp)?;

        let fields = Fields::build(&cf.fields, &cp, msa)?;

        let methods = build_methods(&cf.methods, &cp, msa)?;

        Ok(Self {
            acc_flags,
            this_klass: this_entry.clone(),
            super_klass: super_entry,
            constant_pool: cp,
            interfaces,
            fields,
            methods,
        })
    }
}

#[derive(Debug)]
enum ClassInitState {
    Uninitialized,

    Initializing { owner: JavaThreadID },

    Initialized,

    Erroneous,
}

#[derive(Debug)]
struct ClassInit {
    state: parking_lot::Mutex<ClassInitState>,
    completed: parking_lot::Condvar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassInitAction {
    Claimed,
    AlreadyInitialized,
    RecursiveRequest,
    Erroneous,
}

impl Default for ClassInit {
    fn default() -> Self {
        Self {
            state: parking_lot::Mutex::new(ClassInitState::Uninitialized),
            completed: parking_lot::Condvar::new(),
        }
    }
}

#[derive(Debug)]
pub struct NormalKlass {
    acc_flags: AccFlags,

    this_klass: MSRef<ClassCPEntry>,
    super_klass: Option<MSRef<NormalKlass>>,

    // Points to rust memory space.
    cld: Option<NonNull<ClassLoaderData>>,

    constant_pool: MSBox<[OnceCell<CPEntry>]>,

    interfaces: MSBox<[MSRef<NormalKlass>]>,

    fields: Fields,

    methods: MSBox<[Method]>,

    obj_layout: ObjLayout,

    init: ClassInit,
}

impl NormalKlass {
    pub fn link(
        unlinked: UnlinkedNormalKlass,
        cld: Option<&ClassLoaderData>,
    ) -> ResolveResult<MSBox<Klass>> {
        let msa = match cld {
            Some(x) => &x.ms_allocator,
            None => BootstrapCLD::bs_msa(),
        };

        let obj_layout;
        let super_klass;
        match unlinked.super_klass {
            Some(x) => {
                let super_ref = x.get(cld)?;
                let super_normal = super_ref.as_normal().unwrap();
                super_klass = unsafe { Some(MSRef::from_raw(super_normal.into())) };

                obj_layout = ObjLayout {
                    super_layout: &super_normal.obj_layout,
                    byte_size: super_normal.obj_layout.byte_size + unlinked.fields.instance_size,
                    ptrs_count: unlinked.fields.instance_ptrs_count,
                }
            }

            None => {
                super_klass = None;
                obj_layout = ObjLayout {
                    super_layout: null(),
                    byte_size: unlinked.fields.instance_size,
                    ptrs_count: unlinked.fields.instance_ptrs_count,
                }
            }
        }

        // A linked class keeps direct interfaces as resolved metadata references.
        // Field resolution can then traverse the interface graph without exposing
        // or re-reading symbolic constant-pool entries.
        let interfaces = link_interfaces(&unlinked.interfaces, cld, msa)?;

        let cld_ptr = match cld {
            Some(x) => Some(x.into()),
            None => None,
        };

        let klass = Self {
            acc_flags: unlinked.acc_flags,
            this_klass: unlinked.this_klass,
            super_klass,
            cld: cld_ptr,
            constant_pool: unlinked.constant_pool,
            interfaces,
            fields: unlinked.fields,
            methods: unlinked.methods,
            obj_layout,
            init: ClassInit::default(),
        };

        let boxed = MSBox::new(msa, Klass::Normal(klass));
        boxed.as_normal().unwrap().this_klass.set((&boxed).into());

        Ok(boxed)
    }
}

impl NormalKlass {
    pub fn is_interface(&self) -> bool {
        self.acc_flags.contains(AccFlags::ACC_INTERFACE)
    }
}

impl NormalKlass {
    pub fn obj_layout(&self) -> &ObjLayout {
        &self.obj_layout
    }

    pub fn constant_pool_entry(&self, index: usize) -> Option<&CPEntry> {
        self.constant_pool.get(index)?.get()
    }

    pub fn cld(&self) -> Option<&ClassLoaderData> {
        self.cld.map(|x| unsafe { x.as_ref() })
    }

    pub fn super_klass_ref(&self) -> Option<MSRef<NormalKlass>> {
        self.super_klass.clone()
    }

    /// Acquire this class's initialization state for `owner`.
    ///
    /// This method only coordinates state and waiters.  Deciding whether and
    /// how to execute `<clinit>` belongs to the execution engine.
    pub fn begin_initialization(&self, owner: JavaThreadID) -> ClassInitResult<ClassInitAction> {
        let mut state = self.init.state.lock();
        loop {
            match *state {
                ClassInitState::Uninitialized => {
                    *state = ClassInitState::Initializing { owner };
                    return Ok(ClassInitAction::Claimed);
                }
                ClassInitState::Initializing { owner: current } if current == owner => {
                    return Ok(ClassInitAction::RecursiveRequest);
                }
                ClassInitState::Initializing { .. } => self.init.completed.wait(&mut state),
                ClassInitState::Initialized => {
                    return Ok(ClassInitAction::AlreadyInitialized);
                }
                ClassInitState::Erroneous => return Ok(ClassInitAction::Erroneous),
            }
        }
    }

    pub fn complete_initialization(&self, owner: JavaThreadID) -> ClassInitResult<()> {
        let mut state = self.init.state.lock();
        match *state {
            ClassInitState::Initializing { owner: current } if current == owner => {
                *state = ClassInitState::Initialized;
            }
            _ => return Err(ClassInitError::InvalidTransition),
        }
        self.init.completed.notify_all();
        Ok(())
    }

    /// Release an initialization claim after an internal VM failure. Unlike a
    /// Java exception from `<clinit>`, this permits a later initialization try.
    pub fn abort_initialization(&self, owner: JavaThreadID) -> ClassInitResult<()> {
        let mut state = self.init.state.lock();
        match *state {
            ClassInitState::Initializing { owner: current } if current == owner => {
                *state = ClassInitState::Uninitialized;
            }
            _ => return Err(ClassInitError::InvalidTransition),
        }
        self.init.completed.notify_all();
        Ok(())
    }

    pub fn fail_initialization(&self, owner: JavaThreadID) -> ClassInitResult<()> {
        let mut state = self.init.state.lock();
        match *state {
            ClassInitState::Initializing { owner: current } if current == owner => {
                *state = ClassInitState::Erroneous;
            }
            _ => return Err(ClassInitError::InvalidTransition),
        }
        self.init.completed.notify_all();
        Ok(())
    }
}

impl NormalKlass {
    pub(crate) fn direct_interfaces(&self) -> &[MSRef<NormalKlass>] {
        &self.interfaces
    }

    pub(crate) fn declares_default_method(&self) -> bool {
        self.is_interface()
            && self.methods.iter().any(|method| {
                !method.acc_flags.contains(AccFlags::ACC_ABSTRACT)
                    && !method.acc_flags.contains(AccFlags::ACC_STATIC)
            })
    }

    pub fn find_declared_field_symbol(
        &self,
        name: &SymbolHandle,
        desc: &SymbolHandle,
    ) -> Option<MSRef<Field>> {
        let field = self.fields.find_declared(name, desc)?;
        Some(unsafe { MSRef::from_raw(NonNull::from(field)) })
    }

    pub fn resolve_field_ref(&self, index: usize) -> ResolveResult<ResolvedFieldRef> {
        let entry = self
            .constant_pool_entry(index)
            .ok_or(ResolveError::InvalidCPIndex)?;

        match entry {
            CPEntry::FieldRef(entry) => entry.resolve(self),
            _ => Err(ResolveError::MismatchCPType),
        }
    }

    pub fn read_static_field(&self, field: &Field) -> ExecResult<Vec<Slot>> {
        self.fields.read_static(field)
    }

    pub fn write_static_field(&self, field: &Field, slots: &[Slot]) -> ExecResult<()> {
        self.fields.write_static(field, slots)
    }

    pub fn initialize_static_constant_values(&self) -> ExecResult<()> {
        self.fields.initialize_constant_values()
    }

    pub fn find_declared_method(&self, name: &str, desc: &str) -> Option<MSRef<Method>> {
        let name = SymbolTable::intern(name);
        let desc = SymbolTable::intern(desc);

        self.find_declared_method_symbol(&name, &desc)
    }

    pub fn find_declared_method_symbol(
        &self,
        name: &SymbolHandle,
        desc: &SymbolHandle,
    ) -> Option<MSRef<Method>> {
        let method = self
            .methods
            .iter()
            .find(|method| method.name.equals(name) && method.desc.raw.equals(desc))?;

        Some(unsafe { MSRef::from_raw(NonNull::from(method)) })
    }

    pub fn resolve_method_ref(&self, index: usize) -> ResolveResult<ResolvedMethodRef> {
        let entry = self
            .constant_pool_entry(index)
            .ok_or(ResolveError::InvalidCPIndex)?;

        match entry {
            CPEntry::MethodRef(entry) => entry.resolve(self),
            _ => Err(ResolveError::MismatchCPType),
        }
    }
}
