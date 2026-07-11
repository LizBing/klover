use std::{
    cell::OnceCell, ops::Deref, ptr::{NonNull, null}
};

use crate::{
    class_loader::{
        bootstrap_cld::BootstrapCLD,
        cld::ClassLoaderData,
        ms_api::{MSAllocator, MSBox, MSRef},
    }, class_parser::{
        class_file::ClassFile, cp_info::ConstantPoolInfo, method_info::MethodInfo,
    }, gc_bindings::{obj_layout::ObjLayout, oop_handle::{KLASS_OOP_STORAGE_ID, OOPHandle}}, oops::{
        acc_flags::AccFlags, attr::KlassAttrs, cp_entry::{CPEntry, ClassCPEntry}, field::Field, fields::Fields, klass::Klass, method::Method, resolve_error::{ResolveError, ResolveResult}, symbol_table::SymbolHandle,
    }
};

#[derive(Debug)]
pub struct UnlinkedNormalKlass {
    acc_flags: AccFlags,

    this_klass: MSRef<ClassCPEntry>,

    // Points to rust memory space.
    cld: Option<NonNull<ClassLoaderData>>,

    constant_pool: MSBox<[OnceCell<CPEntry>]>,

    interfaces: MSBox<[MSRef<ClassCPEntry>]>,

    fields: Fields,

    methods: MSBox<[Method]>,

    attrs: KlassAttrs,
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

impl UnlinkedNormalKlass {
    pub fn build(
        cf: ClassFile,
        cld: Option<&ClassLoaderData>,
    ) -> ResolveResult<(Self, Option<MSRef<ClassCPEntry>>)> {
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

        let attrs = KlassAttrs::build(&cf.attrs, &cp, msa)?;

        Ok((Self {
            acc_flags,
            this_klass: this_entry.clone(),
            cld: cld_ptr,
            constant_pool: cp,
            interfaces,
            fields,
            methods,
            attrs
        }, super_entry))
    }
}    

#[derive(Debug)]
pub struct NormalKlass {
    acc_flags: AccFlags,

    this_klass: MSRef<ClassCPEntry>,

    // Points to rust memory space.
    cld: Option<NonNull<ClassLoaderData>>,

    constant_pool: MSBox<[OnceCell<CPEntry>]>,

    interfaces: MSBox<[MSRef<ClassCPEntry>]>,

    fields: Fields,

    methods: MSBox<[Method]>,

    attrs: KlassAttrs,

    super_klass: Option<MSRef<ClassCPEntry>>,

    obj_layout: ObjLayout,
}

impl NormalKlass {
    pub fn link(unlinked: UnlinkedNormalKlass, super_klass: &NormalKlass) -> ResolveResult<MSBox<Self>> {
        unimplemented!()
    }
}

impl NormalKlass {
    pub fn get_obj_layout(&self) -> &ObjLayout {
        unimplemented!()
    }
}
