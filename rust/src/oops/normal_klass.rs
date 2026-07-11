use std::{
    cell::OnceCell, ops::Deref, ptr::{NonNull, null}
};

use crate::{
    class_loader::{
        bootstrap_cld::BootstrapCLD,
        cld::ClassLoaderData,
        ms_api::{MSAllocator, MSBox, MSRef},
    }, class_parser::{
        attr_info::AttrInfo, class_file::ClassFile, cp_info::ConstantPoolInfo, method_info::MethodInfo,
    }, gc_bindings::{obj_layout::ObjLayout, oop_handle::{KLASS_OOP_STORAGE_ID, OOPHandle}}, oops::{
        acc_flags::AccFlags, attr::{BootstrapMethod, build_bs_methods, build_nest_members, build_permitted_subclasses}, cp_entry::{CPEntry, ClassCPEntry}, field::Field, fields::Fields, klass::Klass, method::Method, resolve_error::{ResolveError, ResolveResult}, symbol_table::SymbolHandle,
    }
};

#[derive(Debug)]
pub struct NormalKlass {
    pub mirror: OOPHandle,

    pub acc_flags: AccFlags,

    pub this_klass: MSRef<ClassCPEntry>,
    super_klass: OnceCell<Option<MSRef<NormalKlass>>>, // resolve in cld callsite

    // Points to rust memory space.
    pub cld: Option<NonNull<ClassLoaderData>>,

    constant_pool: MSBox<[OnceCell<CPEntry>]>,

    interfaces: MSBox<[MSRef<ClassCPEntry>]>,

    fields: Fields,

    methods: MSBox<[Method]>,

    /// 对象内存布局描述。`set_super init_fieds` 后可用。
    obj_layout: OnceCell<ObjLayout>,
    
    // attributes

    permitted_subclasses: Option<MSBox<[MSRef<ClassCPEntry>]>>,
    bootstrap_methods: Option<MSBox<[BootstrapMethod]>>,
    nest_host: Option<MSRef<ClassCPEntry>>,
    nest_members: Option<MSBox<[MSRef<ClassCPEntry>]>>
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
    msa: &MSAllocator,
) -> ResolveResult<MSBox<[MSRef<ClassCPEntry>]>> {
    let iface_len = parsed_ifaces.len();
    let uninit = msa.calloc(iface_len);

    for (i, idx) in parsed_ifaces.iter().enumerate() {
        match cp_slice_get(cp_slice, *idx as usize) {
            Some(CPEntry::Class(entry)) => uninit[i].write(entry.into()),
            _ => return Err(ResolveError::MismatchCPType),
        };
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

impl NormalKlass {
    pub fn build(
        cf: ClassFile,
        cld: Option<&ClassLoaderData>,
    ) -> ResolveResult<(MSBox<Klass>, Option<MSRef<ClassCPEntry>>)> {
        let msa = match cld {
            Some(x) => &x.ms_allocator,
            None => BootstrapCLD::bs_msa(),
        };

        let acc_flags = AccFlags::from_bits_truncate(cf.acc_flags);

        let cp = build_cp(&cf.constant_pool, msa)?;

        let this_entry: MSRef<ClassCPEntry> = match cp_slice_get(&cp, cf.this_class as usize) {
            Some(CPEntry::Class(entry)) => entry.into(),
            _ => return Err(ResolveError::MismatchCPType),
        };

        let super_entry = if cf.super_index == 0 {
            None
        } else {
            Some(
                match cp_slice_get(&cp, cf.super_index as usize) {
                    Some(CPEntry::Class(entry)) => entry.into(),
                    _ => return Err(ResolveError::MismatchCPType),
                }
            )
        };

        let interfaces = build_interfaces(&cf.interfaces, &cp, msa)?;

        let fields = Fields::build(&cf.fields, &cp, msa)?;

        let methods = build_methods(&cf.methods, &cp, msa)?;

        let cld_ptr = match cld {
            Some(x) => unsafe { Some(NonNull::new_unchecked(x as *const _ as *mut _)) },
            None => None,
        };

        let mut permitted_subclasses = None;
        let mut bsms = None;
        let mut nest_host = None;
        let mut nest_members = None;
        
        for info in cf.attrs {
            match info {
                AttrInfo::PermittedSubclasses { cp_idxes } =>
                    permitted_subclasses = Some(build_permitted_subclasses(&cp_idxes, &cp, msa)?),

                AttrInfo::BootstrapMethods(x) =>
                    bsms = Some(build_bs_methods(&x, &cp, msa)?),

                AttrInfo::NestHost { cp_idx } =>
                    nest_host = match cp[cp_idx as usize].get() {
                        Some(CPEntry::Class(x)) => Some(x.into()),
                        _ => return Err(ResolveError::MismatchCPType)
                    },

                AttrInfo::NestMembers { cp_idxes } =>
                    nest_members = Some(build_nest_members(&cp_idxes, &cp, msa)?),

                // ignore other attributes
                _ => continue,
            }
        }

        let klass = Self {
            mirror: OOPHandle::new(KLASS_OOP_STORAGE_ID),
            acc_flags,
            this_klass: this_entry.clone(),
            super_klass: OnceCell::new(),
            cld: cld_ptr,
            constant_pool: cp,
            interfaces,
            fields,
            methods,
            obj_layout: OnceCell::new(),

            permitted_subclasses,
            bootstrap_methods: bsms,
            nest_host,
            nest_members
        };

        let boxed = MSBox::new(msa, Klass::Normal(klass));
        this_entry.resolved.set((&boxed).into()).unwrap();

        Ok((boxed, super_entry))
    }
    
    // callsite: cld
    pub fn set_super(&self, s: Option<MSRef<NormalKlass>>) {
        self.super_klass.set(s).unwrap()
    }
    
    // After 'set_super()'
    pub fn cal_object_layout(&self) {
        let (super_layout, super_size) = match self.get_super() {
            Some(super_ref) => {
                let super_layout = super_ref.get_obj_layout();
                (super_layout as *const _, super_layout.byte_size)
            }
            None => (null(), 0)
        };

        let fields = &self.fields;
        
        let layout = ObjLayout {
            super_layout,
            byte_size: super_size + fields.instance_size,
            ptrs_count: fields.instance_ptrs_count
        };

        self.obj_layout.set(layout).unwrap()
    }
}

impl NormalKlass {
    pub fn cp_get(&self, idx: usize) -> Option<&CPEntry> {
        cp_slice_get(&self.constant_pool, idx)
    }

    pub fn find_method(&self, mname: &SymbolHandle, mdesc: &SymbolHandle) -> Option<&Method> {
        for n in self.methods.as_ref() {
            if n.name.equals(mname) && n.desc.raw.equals(mdesc) {
                return Some(n);
            }
        }

        None
    }

    /// 沿继承链查找字段（name + descriptor 同时匹配）。
    /// 返回的字段同时覆盖 instance 与 static，调用方按 acc_flags 区分。
    pub fn find_field(&self, fname: &SymbolHandle, fdesc: &SymbolHandle) -> Option<&Field> {
        let f = &self.fields;
        
        if let Some(x) = f.instance_fields.as_ref() {
            for field in x.as_ref() {
                if field.name.equals(fname) && field.desc.raw.equals(fdesc) {
                    return Some(field);
                }
            }
        }
        
        if let Some(x) = f.static_fields.as_ref() {
            for field in x.as_ref() {
                if field.name.equals(fname) && field.desc.raw.equals(fdesc) {
                    return Some(field);
                }
            }
        }
        
        // 沿继承链向上。
        match self.get_super() {
            Some(s) => s.find_field(fname, fdesc),
            None => None,
        }
    }
}

impl NormalKlass {
    pub fn get_obj_layout(&self) -> &ObjLayout {
        self.obj_layout.get().expect("Obj layout hasn't been initialized.")
    }
    
    pub fn get_super(&self) -> Option<&NormalKlass> {
        match self.super_klass.get().expect("Super hasn't been set.") {
            Some(x) => Some(x.deref()),
            None => None,
        }
    }

    /// 判断 `self` 是否是 `target` 的子类（或就是 `target` 本身）。
    ///
    /// 沿继承链向上，用 MSRef 指针相等判断。  仅支持普通类；
    /// 接口关系（implements）不走继承链，MVP 不支持。
    pub fn is_subclass_of(&self, target: &NormalKlass) -> bool {
        let self_ref = crate::class_loader::ms_api::MSRef::from(self as &NormalKlass);
        let target_ref = crate::class_loader::ms_api::MSRef::from(target as &NormalKlass);
        if self_ref.equals(&target_ref) {
            return true;
        }
        match self.get_super() {
            Some(s) => s.is_subclass_of(target),
            None => false,
        }
    }
}
