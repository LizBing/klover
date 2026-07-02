use std::{
    cell::OnceCell, mem::size_of, ptr::{self, NonNull}, slice, sync::Mutex
};

use crate::{
    class_loader::{cld::ClassLoaderData, ms_box::MSAllocator},
    class_parser::{class_file::ClassFile, cp_info::ConstantPoolInfo, field_info::FieldInfo},
    oops::{
        acc_flags::AccFlags, cp_entry::{CPEntry, ClassCPEntry}, field::Field, method::Method, obj_layout::ObjLayout, oop_handle::{KLASS_OOP_STORAGE_ID, NarrowOOP, OOPHandle}, resolve_error::{ResolveError, ResolveResult}, symbol_table::SymbolHandle
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
    pub name: SymbolHandle,

    // This mutex protects the initialization of these two OnceCell.
    // The linking and initialization of this class happens in the first "new" or the access of a static field.
    super_entry: Mutex<Option<NonNull<ClassCPEntry>>>,
    pub super_klass: OnceCell<Option<NonNull<NormalKlass>>>,
    pub obj_layout: OnceCell<ObjLayout>,

    // Delegation: cld.load_class()
    pub cld: NonNull<ClassLoaderData>,

    constant_pool: NonNull<[Option<CPEntry>]>,

    interfaces: NonNull<[NonNull<ClassCPEntry>]>,
    fields: Fields,
    methods: NonNull<[Method]>,
}

impl NormalKlass {
    pub fn from(
        cf: ClassFile,
        cld: NonNull<ClassLoaderData>,
    ) -> ResolveResult<Self> {
        let acc_flags = AccFlags::from_bits_truncate(cf.acc_flags);
        let msa = unsafe { &cld.as_ref().ms_allocator };

        // 1. Allocate and resolve the constant pool in metaspace.
        let cp_len = cf.constant_pool.len();
        let cp_mem = msa.calloc::<Option<CPEntry>>(size_of::<Option<CPEntry>>(), cp_len);
        let cp_slice = unsafe { slice::from_raw_parts_mut(cp_mem, cp_len) };
        for i in 0..cp_len {
            unsafe {
                ptr::write(&mut cp_slice[i], None);
            }
        }

        // Slot 0 is Unusable (JVM CP is 1-indexed).
        for i in 1..cp_len {
            if matches!(&cf.constant_pool[i], ConstantPoolInfo::Unusable) {
                continue;
            }
            if let Some(entry) = CPEntry::from(i, cp_slice, &cf.constant_pool)? {
                cp_slice[i] = Some(entry);
            }
        }
        let constant_pool = unsafe { NonNull::new_unchecked(cp_slice as *mut [Option<CPEntry>]) };

        // 2. Extract this class's name from its own Class entry.
        let name = match &cp_slice[cf.this_class as usize] {
            Some(CPEntry::Class { entry }) => entry.name.clone(),
            _ => return Err(ResolveError::MismatchCPType),
        };

        // 3. Super class entry.
        let super_entry;
        if cf.super_index == 0 {
            super_entry = None
        } else {
            match unsafe {
                cp_slice[cf.super_index as usize]
                    .as_ref()
                    .unwrap_unchecked()
            } {
                CPEntry::Class { entry } => {
                    super_entry =
                        unsafe { Some(NonNull::new_unchecked(entry as *const _ as *mut _)) }
                }

                _ => return Err(ResolveError::MismatchCPType),
            }
        }

        // 4. Resolve interface Class entries into pointers.
        let iface_len = cf.interfaces.len();
        let iface_mem =
            msa.calloc::<NonNull<ClassCPEntry>>(size_of::<NonNull<ClassCPEntry>>(), iface_len);
        let iface_slice = unsafe { slice::from_raw_parts_mut(iface_mem, iface_len) };

        for (i, idx) in cf.interfaces.iter().enumerate() {
            iface_slice[i] = match &cp_slice[*idx as usize] {
                Some(CPEntry::Class { entry }) => unsafe {
                    NonNull::new_unchecked(entry as *const ClassCPEntry as *mut ClassCPEntry)
                },
                _ => return Err(ResolveError::MismatchCPType),
            };
        }
        let interfaces =
            unsafe { NonNull::new_unchecked(iface_slice as *mut [NonNull<ClassCPEntry>]) };

        // 5. Build fields.
        let fields = Fields::build(&cf.fields, cp_slice, msa)?;

        // 6. Build methods.
        let methods_len = cf.methods.len();
        let methods_mem = msa.calloc::<Method>(size_of::<Method>(), methods_len);
        let methods_slice = unsafe { slice::from_raw_parts_mut(methods_mem, methods_len) };

        for (i, info) in cf.methods.iter().enumerate() {
            unsafe {
                ptr::write(&mut methods_slice[i], Method::from(info, cp_slice, msa)?);
            }
        }
        let methods = unsafe { NonNull::new_unchecked(methods_slice as *mut [Method]) };

        Ok(
            Self {
                mirror: OOPHandle::new(KLASS_OOP_STORAGE_ID),
                acc_flags,
                name,
                super_entry: Mutex::new(super_entry),
                super_klass: OnceCell::new(),
                obj_layout: OnceCell::new(),
                cld,
                constant_pool,
                interfaces,
                fields,
                methods,
            }
        )
    }

    pub fn init(&self) {}
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
}
