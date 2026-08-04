use std::{array, cell::OnceCell, marker::PhantomData, ptr};

use parking_lot::RwLock;

use crate::{
    class_loader::ms_api::{MSAllocator, MSBox},
    class_parser::field_info::FieldInfo,
    engine::{
        exec_error::{ExecError, ExecResult},
        slot::Slot,
    },
    gc_bindings::oop_handle::NObjPtr,
    oops::{
        acc_flags::AccFlags,
        attr::ConstantValue,
        cp_entry::CPEntry,
        desc::{FieldDesc, FieldElemType},
        field::Field,
        oops_errors::ResolveResult,
    },
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

    unsafe { MSBox::from_raw(uninit.assume_init_mut()) }
}

/// 类加载过程中构建的字段信息。
///
/// Instance 字段的 offset 只描述本类声明字段在本类 instance 部分内的局部偏移。
/// 父类 instance 大小由 `NormalKlass::obj_layout` 组合到完整对象布局中。
#[derive(Debug)]
pub struct Fields {
    __: PhantomData<()>,

    pub static_storage: Option<RwLock<MSBox<[u8]>>>,
    pub static_fields: Option<MSBox<[Field]>>,
    pub static_ptrs_count: usize,

    pub instance_size: usize,
    pub instance_fields: Option<MSBox<[Field]>>,
    /// 本类 instance 部分的引用字段数（与 ObjLayout.ptr_count 一致）。
    pub instance_ptrs_count: usize,
}

impl Fields {
    pub(super) fn initialize_constant_values(&self) -> ExecResult<()> {
        let Some(fields) = &self.static_fields else {
            return Ok(());
        };

        // ConstantValue on an instance field is deliberately ignored. Only
        // fields in static_fields participate in this initialization step.
        for field in fields.iter() {
            let Some(value) = &field.constant_value else {
                continue;
            };

            let slots = Self::constant_value_slots(field, value)?;
            self.write_static(field, &slots)?;
        }

        Ok(())
    }

    fn constant_value_slots(field: &Field, value: &ConstantValue) -> ExecResult<Vec<Slot>> {
        if field.desc.dimensions != 0 {
            return Err(ExecError::InvalidConstantValue);
        }

        match (&field.desc.elem, value) {
            (
                FieldElemType::Boolean
                | FieldElemType::Byte
                | FieldElemType::Char
                | FieldElemType::Short
                | FieldElemType::Int,
                ConstantValue::Integer(value),
            ) => Ok(vec![Slot::int(*value)]),
            (FieldElemType::Float, ConstantValue::Float(value)) => Ok(vec![Slot::float(*value)]),
            (FieldElemType::Long, ConstantValue::Long(value)) => {
                Ok(vec![Slot::long_high(*value), Slot::long_low(*value)])
            }
            (FieldElemType::Double, ConstantValue::Double(value)) => {
                Ok(vec![Slot::double_high(*value), Slot::double_low(*value)])
            }
            (FieldElemType::Class { .. }, ConstantValue::String(_))
                if field.desc.raw.utf8() == "Ljava/lang/String;" =>
            {
                Err(ExecError::UnsupportedStringConstantValue)
            }
            _ => Err(ExecError::InvalidConstantValue),
        }
    }

    fn get_bucket<'a>(buckets: &'a mut [Vec<Field>; 5], desc: &FieldDesc) -> &'a mut Vec<Field> {
        if desc.is_ref_type() {
            return &mut buckets[0];
        }

        match desc.byte_size() {
            8 => &mut buckets[1],
            4 => &mut buckets[2],
            2 => &mut buckets[3],
            1 => &mut buckets[4],

            _ => unreachable!(),
        }
    }

    // returns: (byte size, fields, ptrs count)
    fn build_category(
        buckets: &mut [Vec<Field>; 5],
        msa: &MSAllocator,
    ) -> (usize, Option<MSBox<[Field]>>, usize) {
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

                None => break,
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

                None => break,
            }
        } // 4 bytes
        loop {
            match buckets[2].pop() {
                Some(x) => {
                    x.set_offs(byte_size);
                    byte_size += 4;
                    fields_buf.push(x);
                }

                None => break,
            }
        } // 2 bytes
        loop {
            match buckets[3].pop() {
                Some(x) => {
                    x.set_offs(byte_size);
                    byte_size += 2;
                    fields_buf.push(x);
                }

                None => break,
            }
        } // 1 bytes
        loop {
            match buckets[4].pop() {
                Some(x) => {
                    x.set_offs(byte_size);
                    byte_size += 1;
                    fields_buf.push(x);
                }

                None => break,
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

    pub fn build(
        infos: &[FieldInfo],
        cp_slice: &[OnceCell<CPEntry>],
        msa: &MSAllocator,
    ) -> ResolveResult<Self> {
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

        let (instance_size, instance_fields, instance_ptrs_count) =
            Self::build_category(&mut instance_buckets, msa);
        let (s_size, static_fields, static_ptrs_count) =
            Self::build_category(&mut static_buckets, msa);

        let static_storage = if s_size == 0 {
            None
        } else {
            unsafe {
                let uninit = msa.calloc(s_size);
                ptr::write_bytes(uninit.as_mut_ptr(), 0, s_size);
                Some(RwLock::new(MSBox::from_raw(uninit.assume_init_mut())))
            }
        };

        Ok(Self {
            __: PhantomData,

            static_storage,
            static_fields,
            static_ptrs_count,

            instance_size,
            instance_fields,
            instance_ptrs_count,
        })
    }

    pub(super) fn find_declared(
        &self,
        name: &crate::oops::symbol_table::SymbolHandle,
        desc: &crate::oops::symbol_table::SymbolHandle,
    ) -> Option<&Field> {
        self.static_fields
            .iter()
            .flat_map(|fields| fields.iter())
            .chain(self.instance_fields.iter().flat_map(|fields| fields.iter()))
            .find(|field| field.name.equals(name) && field.desc.raw.equals(desc))
    }

    pub(super) fn read_static(&self, field: &Field) -> ExecResult<Vec<Slot>> {
        if !field.acc_flags.contains(AccFlags::ACC_STATIC) {
            return Err(ExecError::IncompatibleStaticFieldAccess);
        }

        let storage = self
            .static_storage
            .as_ref()
            .ok_or(ExecError::InvalidStaticFieldStorage)?
            .read();
        let bytes = Self::field_bytes(&storage, field)?;

        if field.desc.is_ref_type() {
            return Ok(vec![Slot::reference(u32::from_ne_bytes(
                bytes.try_into().unwrap(),
            ))]);
        }

        let slots = match field.desc.elem {
            FieldElemType::Boolean => vec![Slot::int((bytes[0] != 0) as i32)],
            FieldElemType::Byte => vec![Slot::int(i8::from_ne_bytes([bytes[0]]) as i32)],
            FieldElemType::Char => vec![Slot::int(
                u16::from_ne_bytes(bytes.try_into().unwrap()) as i32
            )],
            FieldElemType::Short => vec![Slot::int(
                i16::from_ne_bytes(bytes.try_into().unwrap()) as i32
            )],
            FieldElemType::Int => vec![Slot::int(i32::from_ne_bytes(bytes.try_into().unwrap()))],
            FieldElemType::Float => {
                vec![Slot::float(f32::from_ne_bytes(bytes.try_into().unwrap()))]
            }
            FieldElemType::Long => {
                let value = i64::from_ne_bytes(bytes.try_into().unwrap());
                vec![Slot::long_high(value), Slot::long_low(value)]
            }
            FieldElemType::Double => {
                let value = f64::from_ne_bytes(bytes.try_into().unwrap());
                vec![Slot::double_high(value), Slot::double_low(value)]
            }
            FieldElemType::Class { .. } => unreachable!(),
        };

        Ok(slots)
    }

    pub(super) fn write_static(&self, field: &Field, slots: &[Slot]) -> ExecResult<()> {
        if !field.acc_flags.contains(AccFlags::ACC_STATIC) {
            return Err(ExecError::IncompatibleStaticFieldAccess);
        }

        let value = if field.desc.is_ref_type() {
            let [slot] = slots else {
                return Err(ExecError::InvalidFieldValue);
            };
            slot.as_ref()?.to_ne_bytes().to_vec()
        } else {
            match field.desc.elem {
                FieldElemType::Boolean => {
                    let [slot] = slots else {
                        return Err(ExecError::InvalidFieldValue);
                    };
                    vec![(slot.as_int()? & 1) as u8]
                }
                FieldElemType::Byte => {
                    let [slot] = slots else {
                        return Err(ExecError::InvalidFieldValue);
                    };
                    (slot.as_int()? as i8).to_ne_bytes().to_vec()
                }
                FieldElemType::Char => {
                    let [slot] = slots else {
                        return Err(ExecError::InvalidFieldValue);
                    };
                    (slot.as_int()? as u16).to_ne_bytes().to_vec()
                }
                FieldElemType::Short => {
                    let [slot] = slots else {
                        return Err(ExecError::InvalidFieldValue);
                    };
                    (slot.as_int()? as i16).to_ne_bytes().to_vec()
                }
                FieldElemType::Int => {
                    let [slot] = slots else {
                        return Err(ExecError::InvalidFieldValue);
                    };
                    slot.as_int()?.to_ne_bytes().to_vec()
                }
                FieldElemType::Float => {
                    let [slot] = slots else {
                        return Err(ExecError::InvalidFieldValue);
                    };
                    slot.as_float()?.to_ne_bytes().to_vec()
                }
                FieldElemType::Long => {
                    let [high, low] = slots else {
                        return Err(ExecError::InvalidFieldValue);
                    };
                    Slot::as_long(*high, *low)?.to_ne_bytes().to_vec()
                }
                FieldElemType::Double => {
                    let [high, low] = slots else {
                        return Err(ExecError::InvalidFieldValue);
                    };
                    Slot::as_double(*high, *low)?.to_ne_bytes().to_vec()
                }
                FieldElemType::Class { .. } => unreachable!(),
            }
        };

        let mut storage = self
            .static_storage
            .as_ref()
            .ok_or(ExecError::InvalidStaticFieldStorage)?
            .write();
        let destination = Self::field_bytes_mut(&mut storage, field)?;
        if destination.len() != value.len() {
            return Err(ExecError::InvalidStaticFieldStorage);
        }
        destination.copy_from_slice(&value);
        Ok(())
    }

    fn field_bytes<'a>(storage: &'a [u8], field: &Field) -> ExecResult<&'a [u8]> {
        let start = field.offs();
        let end = start
            .checked_add(field.desc.byte_size())
            .ok_or(ExecError::InvalidStaticFieldStorage)?;
        storage
            .get(start..end)
            .ok_or(ExecError::InvalidStaticFieldStorage)
    }

    fn field_bytes_mut<'a>(storage: &'a mut [u8], field: &Field) -> ExecResult<&'a mut [u8]> {
        let start = field.offs();
        let end = start
            .checked_add(field.desc.byte_size())
            .ok_or(ExecError::InvalidStaticFieldStorage)?;
        storage
            .get_mut(start..end)
            .ok_or(ExecError::InvalidStaticFieldStorage)
    }
}
