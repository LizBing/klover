use std::marker::PhantomData;

use crate::{class_loader::ms_api::{MSAllocator, MSBox}, oops::{desc::FieldDesc, field::Field, oop_handle::NObjPtr}};

#[inline]
fn align(n: usize) -> usize {
    let alignment = size_of::<usize>();
    
    (n + alignment - 1) & !(alignment - 1)
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

    // returns: (storage byte size, fields, ptrs count)
    fn build_catagory(buckets: &mut [Vec<Field>; 5], msa: &MSAllocator) -> (usize, Option<MSBox<[Field]>>, usize) {
        let mut byte_size = 0;
        let mut ptrs_count = 0;
        let mut fields_buf = Vec::new();
        
        // ptrs
        for n in buckets[0] {
            n.set_offs(byte_size);
            byte_size += size_of::<NObjPtr>();
            fields_buf.push(n);

            ptrs_count += 1;
        }

        byte_size = align(byte_size);
        
        // 8 bytes
        for n in buckets[1] {
            n.set_offs(byte_size);
            byte_size += 8;
            fields_buf.push(n);
        }
        
        // 4 bytes
        for n in buckets[2] {
            n.set_offs(byte_size);
            byte_size += 4;
            fields_buf.push(n);
        }
        
        // 2 bytes
        for n in buckets[3] {
            n.set_offs(byte_size);
            byte_size += 2;
            fields_buf.push(n);
        }
        
        // 1 bytes
        for n in buckets[4] {
            n.set_offs(byte_size);
            byte_size += 1;
            fields_buf.push(n);
        }

        byte_size = align(byte_size);

        if fields_buf.len() != 0 {
            let fields = 
        }

        (byte_size, )
    }
    

    fn build(infos: &[FieldInfo], cp_slice: &[Option<CPEntry>], msa: &MSAllocator) -> ResolveResult<Self> {
        let mut instance_buckets = vec![Vec::new(); 5];
        let mut static_buckets = vec![Vec::new(); 5];

        for info in infos {
            let f = Field::from(info, cp)?;
            
            let bucket = if f.acc_flags.contains(AccFlags::ACC_STATIC) {
                Self::get_bucket(&mut static_buckets, &f.desc)
            } else {
                Self::get_bucket(&mut instance_buckets, &f.desc)
            };

            bucket.push(f);
        }

        
    }
}
