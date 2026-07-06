use std::{
    cell::OnceCell, marker::PhantomData, mem::size_of, ops::Deref, ptr::{self, NonNull, null}, slice
};

use crate::{
    class_loader::{
        bootstrap_cld::BootstrapCLD,
        cld::ClassLoaderData,
        ms_api::{MSAllocator, MSBox, MSRef},
    },
    class_parser::{
        class_file::ClassFile, cp_info::ConstantPoolInfo, field_info::FieldInfo,
        method_info::MethodInfo,
    },
    gc_bindings::obj_layout::ObjLayout,
    oops::{
        acc_flags::AccFlags, attr::ConstantValueAttr, cp_entry::{CPEntry, ClassCPEntry}, desc::FieldDesc, field::Field, fields::Fields, klass::Klass, method::Method, oop_handle::{KLASS_OOP_STORAGE_ID, NObjPtr, OOPHandle}, resolve_error::{ResolveError, ResolveResult}, symbol_table::SymbolHandle
    },
};


/// 对象头大小（markword）。
const HEADER_BYTES: usize = 8;

#[derive(Debug)]
pub struct NormalKlass {
    pub mirror: OOPHandle,

    pub acc_flags: AccFlags,

    pub this_klass: MSRef<ClassCPEntry>,
    super_klass: OnceCell<Option<MSRef<NormalKlass>>>, // resolve in cld callsite

    // Points to rust memory space.
    pub cld: Option<NonNull<ClassLoaderData>>,

    constant_pool: MSBox<[Option<CPEntry>]>,

    interfaces: MSBox<[MSRef<ClassCPEntry>]>,

    /// `set_super, init_fields` 后可用；在此之前为 `None`。
    fields: OnceCell<Fields>,

    methods: MSBox<[Method]>,

    /// 对象内存布局描述。`set_super init_fieds` 后可用。
    pub obj_layout: OnceCell<ObjLayout>,
}

fn build_cp<'a>(
    parsed_cp: &[ConstantPoolInfo],
    msa: &MSAllocator,
) -> ResolveResult<MSBox<[Option<CPEntry>]>> {
    let cp_len = parsed_cp.len();
    let cp_mem = msa.calloc::<Option<CPEntry>>(size_of::<Option<CPEntry>>(), cp_len);
    let cp_slice = unsafe { slice::from_raw_parts_mut(cp_mem, cp_len) };
    for i in 0..cp_len {
        unsafe {
            ptr::write(&mut cp_slice[i], None);
        }
    }

    // Slot 0 is Unusable (JVM CP is 1-indexed).
    for i in 1..cp_len {
        if matches!(parsed_cp[i], ConstantPoolInfo::Unusable) {
            continue;
        }
        if let Some(entry) = CPEntry::from(i, cp_slice, parsed_cp)? {
            cp_slice[i] = Some(entry);
        }
    }

    unsafe { Ok(MSBox::from_raw(cp_slice)) }
}

pub fn cp_slice_get(cp_slice: &[Option<CPEntry>], idx: usize) -> Option<&CPEntry> {
    cp_slice[idx].as_ref()
}

fn build_interfaces(
    parsed_ifaces: &[u16],
    cp_slice: &[Option<CPEntry>],
    msa: &MSAllocator,
) -> ResolveResult<MSBox<[MSRef<ClassCPEntry>]>> {
    let iface_len = parsed_ifaces.len();
    let iface_mem = msa.calloc::<MSRef<ClassCPEntry>>(size_of::<MSRef<ClassCPEntry>>(), iface_len);
    let iface_slice = unsafe { slice::from_raw_parts_mut(iface_mem, iface_len) };

    for (i, idx) in parsed_ifaces.iter().enumerate() {
        iface_slice[i] = match cp_slice_get(cp_slice, *idx as usize) {
            Some(CPEntry::Class(entry)) => entry.into(),
            _ => return Err(ResolveError::MismatchCPType),
        };
    }

    unsafe { Ok(MSBox::from_raw(iface_slice)) }
}

fn build_methods(
    parsed_methods: &[MethodInfo],
    cp_slice: &[Option<CPEntry>],
    msa: &MSAllocator,
) -> ResolveResult<MSBox<[Method]>> {
    let methods_len = parsed_methods.len();
    let methods_mem = msa.calloc::<Method>(size_of::<Method>(), methods_len);
    let methods_slice = unsafe { slice::from_raw_parts_mut(methods_mem, methods_len) };

    for (i, info) in parsed_methods.iter().enumerate() {
        unsafe {
            ptr::write(&mut methods_slice[i], Method::from(info, cp_slice, msa)?);
        }
    }

    unsafe { Ok(MSBox::from_raw(methods_slice)) }
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
                    Some(CPEntry::Class(entry)) => entry,
                    _ => return Err(ResolveError::MismatchCPType),
                }
                .into(),
            )
        };

        let interfaces = build_interfaces(&cf.interfaces, &cp, msa)?;

        let methods = build_methods(&cf.methods, &cp, msa)?;

        let cld_ptr = match cld {
            Some(x) => unsafe { Some(NonNull::new_unchecked(x as *const _ as *mut _)) },
            None => None,
        };

        let klass = Self {
            mirror: OOPHandle::new(KLASS_OOP_STORAGE_ID),
            acc_flags,
            this_klass: this_entry.clone(),
            super_klass: OnceCell::new(),
            cld: cld_ptr,
            constant_pool: cp,
            interfaces,
            fields: OnceCell::new(),
            methods,
            obj_layout: OnceCell::new()
        };

        let boxed = MSBox::new(msa, Klass::Normal(klass));
        this_entry.resolved.set((&boxed).into());

        Ok((boxed, super_entry))
    }

    pub fn cal_object_layout(&self) {
        let super_layout = match self.get_super() {
            Some(super_ref) => super_ref.get_obj_layout(),
            None => null()
        };

        let fields = self.get_fields();
        
        let layout = ObjLayout {
            super_layout,
            byte_size: fields.instance_size,
            ptrs_count: fields.instance_ptrs_count
        };

        self.obj_layout.set(layout).unwrap()
    }

    // callsite: cld
    pub fn set_super(&self, s: Option<MSRef<NormalKlass>>) {
        self.super_klass.set(s).unwrap()
    }
}

impl NormalKlass {
    pub fn cp_get(&self, idx: usize) -> &CPEntry {
        unsafe { self.constant_pool.as_ref()[idx].as_ref().unwrap_unchecked() }
    }

    /// 返回已 finalize 的 `Fields`。仅在 `set_super` 后可用。
    pub fn get_fields(&self) -> &Fields {
        self.fields.get().expect("fields not finalized yet")
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
        let f = self.get_fields();
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
    pub fn get_msa(&self) -> &MSAllocator {
        match self.cld {
            Some(x) => unsafe { &x.as_ref().ms_allocator },
            None => BootstrapCLD::bs_msa()
        }
    }
    
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
