use crate::{
    class_parser::{FieldAccessFlags, FieldInfo, LoadableConstant},
    gc_bindings::{
        obj_layout::{OBJECT_ALIGNMENT, ObjLayout},
        oop_hierarchy::NObjPtr,
    },
    oops::{
        desc::{FieldDesc, FieldGroup},
        jvalue::{
            JBoolean, JByte, JChar, JDouble, JFloat, JInt, JLong, JPrimitive, JShort, JValue,
        },
        symbol_table::{SymbolHandle, SymbolTable},
    },
    runtime::ms_api::{MsAllocator, MsBox},
};
use std::{cell::UnsafeCell, sync::Arc};

#[derive(Debug)]
pub struct Field {
    index: usize,
    constant_value: Option<Arc<LoadableConstant>>,
    pub name: SymbolHandle,
    pub desc: FieldDesc,
    acc_flags: FieldAccessFlags,
    /// Set at link time, relative to the object or the declaring class's static block.
    pub offset: usize,
}

impl Field {
    pub fn is_public(&self) -> bool {
        self.acc_flags.contains(FieldAccessFlags::PUBLIC)
    }

    pub fn is_private(&self) -> bool {
        self.acc_flags.contains(FieldAccessFlags::PRIVATE)
    }

    pub fn is_protected(&self) -> bool {
        self.acc_flags.contains(FieldAccessFlags::PROTECTED)
    }

    pub fn is_static(&self) -> bool {
        self.acc_flags.contains(FieldAccessFlags::STATIC)
    }

    pub fn is_final(&self) -> bool {
        self.acc_flags.contains(FieldAccessFlags::FINAL)
    }

    pub fn is_volatile(&self) -> bool {
        self.acc_flags.contains(FieldAccessFlags::VOLATILE)
    }

    pub fn size(&self) -> usize {
        self.desc.field_group().size()
    }

    pub(super) fn build_declared<'dc>(
        infos: &[FieldInfo],
        msa: &'dc MsAllocator,
    ) -> MsBox<'dc, [Field]> {
        let mut declared = MsBox::new_uninit_slice(msa, infos.len());
        for (index, (slot, info)) in declared.iter_mut().zip(infos).enumerate() {
            slot.write(Field {
                index,
                constant_value: info.attributes.constant_value.clone(),
                name: SymbolTable::intern(&info.name),
                desc: FieldDesc::from(&info.descriptor),
                acc_flags: info.access_flags,
                offset: 0,
            });
        }
        // Each input initialized its corresponding slot.
        unsafe { declared.assume_init() }
    }
}

#[derive(Debug)]
pub struct Fields<'dc> {
    declared: MsBox<'dc, [Field]>,
    pub instance: ObjLayout,
    statics: StaticFields<'dc>,
}

#[derive(Debug)]
struct StaticFields<'dc> {
    layout: ObjLayout,
    storage: MsBox<'dc, [UnsafeCell<u64>]>,
}

// SAFETY: Only the immutable layout and owning Fields metadata are exposed.
// Each published slot is accessed atomically with its declared width. Padding
// is private, and no u64 reference into a storage word is ever created.
unsafe impl Sync for StaticFields<'_> {}

impl<'dc> StaticFields<'dc> {
    fn new(layout: ObjLayout, msa: &'dc MsAllocator) -> Self {
        let mut storage = MsBox::new_uninit_slice(msa, layout.byte_size / size_of::<u64>());
        for word in storage.iter_mut() {
            word.write(UnsafeCell::new(0));
        }
        Self {
            layout,
            // SAFETY: Every storage word was initialized above.
            storage: unsafe { storage.assume_init() },
        }
    }

    fn slot_address(&self, field: &Field) -> *mut u8 {
        let word_size = size_of::<u64>();
        let word = &self.storage[field.offset / word_size];
        word.get()
            .cast::<u8>()
            .wrapping_add(field.offset % word_size)
    }

    // SAFETY: The caller must pass a declared static field from this block.
    unsafe fn load<const VOLATILE: bool>(&self, field: &Field) -> JValue {
        let addr = self.slot_address(field);
        // SAFETY: Linking assigns nonoverlapping slots aligned for their width.
        unsafe {
            match field.desc.raw().utf8().as_bytes()[0] {
                b'Z' => JValue::Boolean(JBoolean::load::<VOLATILE>(addr.cast())),
                b'B' => JValue::Byte(JByte::load::<VOLATILE>(addr.cast())),
                b'C' => JValue::Char(JChar::load::<VOLATILE>(addr.cast())),
                b'S' => JValue::Short(JShort::load::<VOLATILE>(addr.cast())),
                b'I' => JValue::Int(JInt::load::<VOLATILE>(addr.cast())),
                b'F' => JValue::Float(JFloat::load::<VOLATILE>(addr.cast())),
                b'J' => JValue::Long(JLong::load::<VOLATILE>(addr.cast())),
                b'D' => JValue::Double(JDouble::load::<VOLATILE>(addr.cast())),
                b'L' | b'[' => JValue::Ref(NObjPtr::load::<VOLATILE>(addr.cast())),
                _ => unreachable!("validated field descriptor"),
            }
        }
    }

    // SAFETY: The field must belong to this block and the value must match its type.
    unsafe fn store<const VOLATILE: bool>(&self, field: &Field, value: JValue) {
        let addr = self.slot_address(field);
        // SAFETY: Linking assigns nonoverlapping slots aligned for their width.
        unsafe {
            match value {
                JValue::Boolean(JBoolean(value)) => {
                    JBoolean::store::<VOLATILE>(addr.cast(), JBoolean(value & 1));
                }
                JValue::Byte(value) => JByte::store::<VOLATILE>(addr.cast(), value),
                JValue::Char(value) => JChar::store::<VOLATILE>(addr.cast(), value),
                JValue::Short(value) => JShort::store::<VOLATILE>(addr.cast(), value),
                JValue::Int(value) => JInt::store::<VOLATILE>(addr.cast(), value),
                JValue::Float(value) => JFloat::store::<VOLATILE>(addr.cast(), value),
                JValue::Long(value) => JLong::store::<VOLATILE>(addr.cast(), value),
                JValue::Double(value) => JDouble::store::<VOLATILE>(addr.cast(), value),
                JValue::Ref(value) => NObjPtr::store::<VOLATILE>(addr.cast(), value),
            }
        }
    }
}

impl<'dc> Fields<'dc> {
    pub(super) fn build(
        mut declared: MsBox<'dc, [Field]>,
        begin_offs: usize,
        msa: &'dc MsAllocator,
    ) -> Self {
        let instance = Self::layout_part(&mut declared, begin_offs, false);
        let static_layout = Self::layout_part(&mut declared, 0, true);
        Self {
            declared,
            instance,
            statics: StaticFields::new(static_layout, msa),
        }
    }

    fn layout_part(declared: &mut [Field], begin_offs: usize, is_static: bool) -> ObjLayout {
        use FieldGroup::*;
        let mut cursor = begin_offs;
        let mut ptrs_count = 0;
        // Keep metadata in declaration order. Only the slot offsets are grouped.
        for group in [Reference, EightBytes, FourBytes, TwoBytes, OneByte] {
            for field in declared
                .iter_mut()
                .filter(|field| field.is_static() == is_static && field.desc.field_group() == group)
            {
                cursor = cursor.next_multiple_of(group.size());
                field.offset = cursor;
                cursor += group.size();
            }
            if group == Reference {
                ptrs_count = (cursor - begin_offs) / group.size();
            }
        }

        ObjLayout {
            begin_offs,
            byte_size: cursor.next_multiple_of(OBJECT_ALIGNMENT),
            ptrs_count,
        }
    }

    fn check_static_field(&self, field: &Field) {
        assert!(
            field.is_static()
                && self
                    .declared
                    .get(field.index)
                    .is_some_and(|declared| std::ptr::eq(declared, field)),
            "static field must belong to this class"
        );
    }

    pub(crate) fn load_static(&self, field: &Field) -> JValue {
        self.check_static_field(field);
        // SAFETY: The owner and kind check above identifies a slot in this block.
        unsafe {
            if field.is_volatile() {
                self.statics.load::<true>(field)
            } else {
                self.statics.load::<false>(field)
            }
        }
    }

    pub(crate) fn store_static(&self, field: &Field, value: JValue) {
        self.check_static_field(field);
        assert!(
            field.desc.matches_value(&value),
            "value must match field type"
        );
        // SAFETY: The checks above identify the slot and its exact storage type.
        unsafe {
            if field.is_volatile() {
                self.statics.store::<true>(field, value);
            } else {
                self.statics.store::<false>(field, value);
            }
        }
    }

    pub(crate) fn initialize_constants(&self) -> Result<(), &Field> {
        for field in self.declared.iter().filter(|field| field.is_static()) {
            let Some(value) = field.constant_value.as_deref() else {
                continue;
            };
            let value = match value {
                LoadableConstant::Integer(value) => match field.desc.raw().utf8().as_bytes()[0] {
                    b'Z' => JValue::Boolean(JBoolean((*value & 1) as u8)),
                    b'B' => JValue::Byte(JByte(*value as i8)),
                    b'C' => JValue::Char(JChar(*value as u16)),
                    b'S' => JValue::Short(JShort(*value as i16)),
                    b'I' => JValue::Int(JInt(*value)),
                    _ => unreachable!("validated integer constant"),
                },
                LoadableConstant::Float(value) => JValue::Float(JFloat(*value)),
                LoadableConstant::Long(value) => JValue::Long(JLong(*value)),
                LoadableConstant::Double(value) => JValue::Double(JDouble(*value)),
                _ => return Err(field),
            };
            self.store_static(field, value);
        }
        Ok(())
    }

    pub fn static_layout(&self) -> &ObjLayout {
        &self.statics.layout
    }

    pub fn declared(&self) -> &[Field] {
        &self.declared
    }
}
