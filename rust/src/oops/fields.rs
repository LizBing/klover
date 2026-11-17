use crate::class_parser::LoadableConstant;
use crate::{
    class_parser::{FieldAccessFlags, FieldInfo},
    gc_bindings::obj_layout::{OBJECT_ALIGNMENT, ObjLayout},
    oops::{
        desc::{FieldDesc, FieldGroup},
        symbol_table::{SymbolHandle, SymbolTable},
    },
    runtime::ms_api::{MsAllocator, MsBox},
};
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use std::mem::MaybeUninit;
use std::sync::Arc;

#[derive(Debug)]
pub struct Field {
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
        for (slot, info) in declared.iter_mut().zip(infos) {
            slot.write(Field {
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
    pub statics: StaticFields<'dc>,
}

#[derive(Debug)]
pub struct StaticFields<'dc> {
    pub layout: ObjLayout,
    storage: Mutex<MsBox<'dc, [u8]>>,
}

impl<'dc> StaticFields<'dc> {
    fn new(layout: ObjLayout, msa: &'dc MsAllocator) -> Self {
        // Metaspace aligns every allocation to at least eight bytes. An empty
        // slice consumes no arena space.
        let mut bytes = MsBox::new_uninit_slice(msa, layout.byte_size);
        bytes.fill(MaybeUninit::new(0));
        Self {
            layout,
            // Every byte, including alignment padding, was initialized above.
            storage: Mutex::new(unsafe { bytes.assume_init() }),
        }
    }

    /// Borrow the live bytes under the block's lock for field access or root scanning.
    pub fn bytes(&self) -> MappedMutexGuard<'_, [u8]> {
        MutexGuard::map(self.storage.lock(), |bytes| bytes.as_mut())
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

    pub(crate) fn initialize_constants(&self) -> Result<(), &Field> {
        let mut storage = self.statics.bytes();
        for field in self.declared.iter().filter(|field| field.is_static()) {
            let Some(value) = field.constant_value.as_deref() else {
                continue;
            };
            let mut encoded = [0u8; 8];
            match value {
                LoadableConstant::Integer(value) => match field.size() {
                    1 => encoded[0] = *value as u8,
                    2 => encoded[..2].copy_from_slice(&(*value as u16).to_ne_bytes()),
                    4 => encoded[..4].copy_from_slice(&value.to_ne_bytes()),
                    _ => unreachable!("validated integer constant"),
                },
                LoadableConstant::Float(value) => {
                    encoded[..4].copy_from_slice(&value.to_ne_bytes())
                }
                LoadableConstant::Long(value) => encoded.copy_from_slice(&value.to_ne_bytes()),
                LoadableConstant::Double(value) => encoded.copy_from_slice(&value.to_ne_bytes()),
                _ => return Err(field),
            }
            storage[field.offset..field.offset + field.size()]
                .copy_from_slice(&encoded[..field.size()]);
        }
        Ok(())
    }

    pub fn declared(&self) -> &[Field] {
        &self.declared
    }
}
