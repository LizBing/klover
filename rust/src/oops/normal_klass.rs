use std::{
    cell::OnceCell,
    mem::size_of,
    ops::DerefMut,
    ptr::{self, NonNull},
    slice,
    sync::atomic::Ordering
};

use crate::{
    class_loader::{
        bootstrap_cld::BootstrapCLD,
        cld::ClassLoaderData,
        ms_api::{MSAllocator, MSBox}
    },
    class_parser::{
        class_file::ClassFile,
        cp_info::ConstantPoolInfo,
        field_info::FieldInfo,
        method_info::MethodInfo
    },
    oops::{
        acc_flags::AccFlags,
        cp_entry::{CPEntry, ClassCPEntry},
        field::Field, klass::Klass,
        method::Method,
        oop_handle::{KLASS_OOP_STORAGE_ID, NarrowOOP, OOPHandle},
        resolve_error::{ResolveError, ResolveResult},
        symbol_table::SymbolHandle
    },
};

/// Copy a `Vec<T>` into metaspace memory and return a `NonNull<[T]>` slice.
fn allocate_slice_from_vec<T>(msa: &MSAllocator, vec: Vec<T>) -> NonNull<[T]> {
    let len = vec.len();
    let mem = msa.calloc::<T>(size_of::<T>(), len);
    let slice = unsafe { slice::from_raw_parts_mut(mem, len) };
    for (i, item) in vec.into_iter().enumerate() {
        unsafe { slice.as_mut_ptr().add(i).write(item) };
    }
    unsafe { NonNull::new_unchecked(slice as *mut [T]) }
}

struct Fields {
    static_ptr_count: usize,
    static_fields: NonNull<[Field]>,
    static_payload_size: usize,

    ptr_count: usize,
    fields: NonNull<[Field]>,
    size: usize,
}

impl Fields {
    fn build(
        infos: &[FieldInfo],
        cp: &[Option<CPEntry>],
        msa: &MSAllocator,
    ) -> ResolveResult<Self> {
        let align = size_of::<usize>();
        let ptr_size = size_of::<NarrowOOP>();

        // Size bucket: 0 = pointer (NarrowOOP), 1 = 8B, 2 = 4B, 3 = 2B, 4 = 1B.
        let bucket = |f: &Field| -> usize {
            match f.desc.byte_size() {
                s if s == ptr_size => 0,
                8 => 1,
                4 => 2,
                2 => 3,
                _ => 4,
            }
        };

        // Separate static and instance fields into size buckets.
        let mut static_buckets: [Vec<Field>; 5] = Default::default();
        let mut instance_buckets: [Vec<Field>; 5] = Default::default();

        for info in infos {
            let f = Field::from(info, cp)?;
            let cat = bucket(&f);
            if f.acc_flags.contains(AccFlags::ACC_STATIC) {
                static_buckets[cat].push(f);
            } else {
                instance_buckets[cat].push(f);
            }
        }

        // Helper: compute offsets in layout order and drain buckets into a Vec.
        fn layout_group(
            buckets: &mut [Vec<Field>; 5],
            ptr_size: usize,
            align: usize,
        ) -> (Vec<Field>, usize, usize) {
            let mut ordered = Vec::new();
            let ptr_count = buckets[0].len();
            let mut offset = 0usize;

            // 0. Pointer fields (NarrowOOP).
            for f in buckets[0].iter_mut() {
                f.set_offs(offset);
                offset += ptr_size;
            }
            // Align after pointer fields to word boundary.
            offset = (offset + align - 1) & !(align - 1);

            // 1. 8-byte fields.
            for f in buckets[1].iter_mut() {
                f.set_offs(offset);
                offset += 8;
            }

            // 2. 4-byte fields.
            for f in buckets[2].iter_mut() {
                f.set_offs(offset);
                offset += 4;
            }

            // 3. 2-byte fields.
            for f in buckets[3].iter_mut() {
                f.set_offs(offset);
                offset += 2;
            }

            // 4. 1-byte fields.
            for f in buckets[4].iter_mut() {
                f.set_offs(offset);
                offset += 1;
            }

            // Drain all buckets into the ordered vec.
            for bucket in buckets.iter_mut() {
                ordered.append(bucket);
            }

            (ordered, ptr_count, offset)
        }

        // Static fields.
        let (static_ordered, static_ptr_count, static_payload_size) =
            layout_group(&mut static_buckets, ptr_size, align);

        let static_fields = allocate_slice_from_vec(msa, static_ordered);

        // Instance fields.
        let (instance_ordered, ptr_count, size) =
            layout_group(&mut instance_buckets, ptr_size, align);

        let fields = allocate_slice_from_vec(msa, instance_ordered);

        Ok(Self {
            static_ptr_count,
            static_fields,
            static_payload_size,
            ptr_count,
            fields,
            size,
        })
    }
}

pub struct NormalKlass {
    pub mirror: OOPHandle,

    pub acc_flags: AccFlags,
    // pub name: SymbolHandle,

    pub this_klass: NonNull<ClassCPEntry>,
    super_klass: OnceCell<Option<NonNull<NormalKlass>>>, // resolve in cld callsite
    
    pub cld: Option<NonNull<ClassLoaderData>>,

    constant_pool: NonNull<[Option<CPEntry>]>,

    interfaces: NonNull<[NonNull<ClassCPEntry>]>,
    fields: Fields,
    methods: NonNull<[Method]>,
}

fn build_cp<'a>(parsed_cp: &[ConstantPoolInfo], msa: &MSAllocator) -> ResolveResult<&'a [Option<CPEntry>]> {
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
    
    Ok(cp_slice)
}

pub fn cp_slice_get(cp_slice: &[Option<CPEntry>], idx: usize) -> &CPEntry {
    unsafe { cp_slice[idx].as_ref().unwrap_unchecked() }
}

fn build_interfaces(parsed_ifaces: &[u16], cp_slice: &[Option<CPEntry>], msa: &MSAllocator) -> ResolveResult<NonNull<[NonNull<ClassCPEntry>]>> {
    let iface_len = parsed_ifaces.len();
    let iface_mem =
        msa.calloc::<NonNull<ClassCPEntry>>(size_of::<NonNull<ClassCPEntry>>(), iface_len);
    let iface_slice = unsafe { slice::from_raw_parts_mut(iface_mem, iface_len) };

    for (i, idx) in parsed_ifaces.iter().enumerate() {
        iface_slice[i] = match cp_slice_get(cp_slice, *idx as usize) {
            CPEntry::Class { entry } => unsafe {
                NonNull::new_unchecked(entry as *const ClassCPEntry as *mut ClassCPEntry)
            },
            _ => return Err(ResolveError::MismatchCPType),
        };
    }

    unsafe { Ok(NonNull::new_unchecked(iface_slice as *mut [NonNull<ClassCPEntry>])) }
}

fn build_methods(parsed_methods: &[MethodInfo], cp_slice: &[Option<CPEntry>], msa: &MSAllocator) -> ResolveResult<NonNull<[Method]>> {
    let methods_len = parsed_methods.len();
    let methods_mem = msa.calloc::<Method>(size_of::<Method>(), methods_len);
    let methods_slice = unsafe { slice::from_raw_parts_mut(methods_mem, methods_len) };

    for (i, info) in parsed_methods.iter().enumerate() {
        unsafe {
            ptr::write(&mut methods_slice[i], Method::from(info, cp_slice, msa)?);
        }
    }
    
    unsafe { Ok(NonNull::new_unchecked(methods_slice as *mut [Method])) }
}

impl NormalKlass {
    pub fn build<'a>(
        cf: ClassFile,
        cld: Option<&ClassLoaderData>
    ) -> ResolveResult<(MSBox<Klass>, Option<&'a ClassCPEntry>)> {
        let msa = match cld {
            Some(x) => &x.ms_allocator,
            None => BootstrapCLD::bs_msa()
        };
        
        let acc_flags = AccFlags::from_bits_truncate(cf.acc_flags);

        let cp_slice = build_cp(&cf.constant_pool, msa)?;
        let cp = unsafe { NonNull::new_unchecked(cp_slice as *const _ as *mut _) };

        let this_entry = match cp_slice_get(cp_slice, cf.this_class as usize) {
            CPEntry::Class { entry } => entry,
            _ => return Err(ResolveError::MismatchCPType)
        };
        
        let super_entry = if cf.super_index == 0 {
            None
        } else {
            Some(match cp_slice_get(cp_slice, cf.super_index as usize) {
                CPEntry::Class { entry } => entry,
                _ => return Err(ResolveError::MismatchCPType)
            })
        };

        let interfaces = build_interfaces(&cf.interfaces, cp_slice, msa)?;

        // 5. Build fields.
        let fields = Fields::build(&cf.fields, cp_slice, msa)?;

        // 6. Build methods.
        let methods = build_methods(&cf.methods, cp_slice, msa)?;

        let cld_ptr = match cld {
            Some(x) => unsafe { Some(NonNull::new_unchecked(x as *const _ as *mut _)) },
            None => None
        };
        
        let klass = Self {
            mirror: OOPHandle::new(KLASS_OOP_STORAGE_ID),
            acc_flags,
            this_klass: unsafe { NonNull::new_unchecked(this_entry as *const _ as *mut _) },
            super_klass: OnceCell::new(),
            cld: cld_ptr,
            constant_pool: cp,
            interfaces,
            fields,
            methods
        };

        let mut boxed = MSBox::new(msa, Klass::Normal(klass));
        this_entry.resolved.store(boxed.deref_mut(), Ordering::Relaxed);

        Ok((boxed, super_entry))
    }

    // callsite: cld
    pub fn set_super(&self, s: Option<NonNull<NormalKlass>>) {
        self.super_klass.set(s).unwrap()
    }
}
    
impl NormalKlass {
    pub fn cp_get(&self, idx: usize) -> &CPEntry {
        unsafe { self.constant_pool.as_ref()[idx].as_ref().unwrap_unchecked() }
    }

    pub fn find_method(&self, mname: &SymbolHandle, mdesc: &SymbolHandle) -> Option<&Method> {
        for n in unsafe { self.methods.as_ref() } {
            if n.name.equals(mname) && n.desc.raw.equals(mdesc) {
                return Some(n);
            }
        }

        None
    }

    pub fn get_super(&self) -> Option<&NormalKlass> {
        let ptr = self.super_klass.get().unwrap();
        match ptr {
            Some(x) => unsafe { Some(x.as_ref()) },
            None => None
        }
    }
}
