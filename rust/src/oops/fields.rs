use std::{array, marker::PhantomData, ptr};

use crate::{
    class_loader::ms_api::{MSAllocator, MSBox},
    class_parser::field_info::FieldInfo,
    oops::{
        acc_flags::AccFlags,
        cp_entry::CPEntry,
        desc::FieldDesc,
        field::Field,
        oop_handle::NObjPtr,
        resolve_error::ResolveResult
    }
};

#[inline]
fn align(n: usize) -> usize {
    let alignment = size_of::<usize>();
    
    (n + alignment - 1) & !(alignment - 1)
}

fn allocate_slice_from_vec<T>(msa: &MSAllocator, vec: Vec<T>) -> MSBox<[T]> {
    let len = vec.len();
    let uninit = msa.calloc(len);

    for (i, v) in vec.into_iter().enumerate() {
        uninit[i].write(v);
    }

    unsafe {
        MSBox::from_raw(uninit.assume_init_mut())
    }
}


/// 类加载完成、`set_super` 调用后构建的完整字段信息。
///
/// 此时 instance 字段的 offset 已含父类偏移，可用于 `getfield`/`putfield`。
#[derive(Debug)]
pub struct Fields {
    __: PhantomData<()>,
    
    pub static_storage: Option<MSBox<[u8]>>,
    pub static_fields: Option<MSBox<[Field]>>,
    pub static_ptrs_count: usize,

    pub instance_size: usize,
    pub instance_fields: Option<MSBox<[Field]>>,
    /// 本类 instance 部分的引用字段数（与 ObjLayout.ptr_count 一致）。
    pub instance_ptrs_count: usize,
}

impl Fields {
    fn get_bucket<'a>(buckets: &'a mut [Vec<Field>; 5], desc: &FieldDesc) -> &'a mut Vec<Field> {
        if desc.is_ref_type() { return &mut buckets[0] }

        match desc.byte_size() {
            8 => &mut buckets[1],
            4 => &mut buckets[2],
            2 => &mut buckets[3],
            1 => &mut buckets[4],

            _ => unreachable!()
        }
    }

    // returns: (byte size, fields, ptrs count)
    fn build_catagory(buckets: &mut [Vec<Field>; 5], msa: &MSAllocator) -> (usize, Option<MSBox<[Field]>>, usize) {
        let mut byte_size = 0;
        let mut ptrs_count = 0;
        let mut fields_buf = Vec::new();
        
        // ptrs
        loop {
            match buckets[0].pop() {
                Some(x) => {
                    x.set_offs(byte_size);
                    byte_size += size_of::<NObjPtr>();
                    fields_buf.push(x);
                    
                    ptrs_count += 1;
                }

                None => break
            }
        }
        
        byte_size = align(byte_size);
        
        // 8 bytes
        loop {
            match buckets[1].pop() {
                Some(x) => {
                    x.set_offs(byte_size);
                    byte_size += 8;
                    fields_buf.push(x);
                }

                None => break
            }
        }// 4 bytes
        loop {
            match buckets[2].pop() {
                Some(x) => {
                    x.set_offs(byte_size);
                    byte_size += 4;
                    fields_buf.push(x);
                }

                None => break
            }
        }// 2 bytes
        loop {
            match buckets[3].pop() {
                Some(x) => {
                    x.set_offs(byte_size);
                    byte_size += 2;
                    fields_buf.push(x);
                }

                None => break
            }
        }// 1 bytes
        loop {
            match buckets[4].pop() {
                Some(x) => {
                    x.set_offs(byte_size);
                    byte_size += 1;
                    fields_buf.push(x);
                }

                None => break
            }
        }

        byte_size = align(byte_size);

        if fields_buf.len() != 0 {
            let fields = allocate_slice_from_vec(msa, fields_buf);
            (byte_size, Some(fields), ptrs_count)
        } else {
            (0, None, 0)
        }
    }
    

    pub fn build(infos: &[FieldInfo], cp_slice: &[Option<CPEntry>], msa: &MSAllocator) -> ResolveResult<Self> {
        let mut instance_buckets = array::from_fn(|_| Vec::new());
        let mut static_buckets = array::from_fn(|_| Vec::new());

        for info in infos {
            let f = Field::from(info, cp_slice)?;
            
            let bucket = if f.acc_flags.contains(AccFlags::ACC_STATIC) {
                Self::get_bucket(&mut static_buckets, &f.desc)
            } else {
                Self::get_bucket(&mut instance_buckets, &f.desc)
            };

            bucket.push(f);
        }

        let (instance_size, instance_fields, instance_ptrs_count) = Self::build_catagory(&mut instance_buckets, msa);
        let (s_size, static_fields, static_ptrs_count) = Self::build_catagory(&mut static_buckets, msa);

        let static_storage = if s_size == 0 {
            None
        } else {
            unsafe {
                let uninit = msa.calloc(s_size);
                ptr::write_bytes(uninit.as_mut_ptr(), 0, s_size);
                Some(MSBox::from_raw(uninit.assume_init_mut()))
            }
        };

        Ok(Self {
            __: PhantomData,

            static_storage,
            static_fields,
            static_ptrs_count,

            instance_size,
            instance_fields,
            instance_ptrs_count
        })
    }
}
