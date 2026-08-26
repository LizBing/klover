use std::cell::OnceCell;

use crate::class_parser::{cp_info::ConstantPoolInfo, desc_image::{FieldDescImage, MethodDescImage}, parse_error::{CPKind, ImageError, ImageResult}};

#[derive(Debug)]
pub enum MethodHandleImage {
    GetField(usize),           // 1
    GetStatic(usize),          // 2
    PutField(usize),           // 3
    PutStatic(usize),          // 4
    InvokeVirtual(usize),      // 5
    InvokeStatic(usize),       // 6
    InvokeSpecial(usize),      // 7
    NewInvokeSpecial(usize),   // 8
    InvokeInterface(usize),    // 9
}

use self::MethodHandleImage::*;
impl MethodHandleImage {
    fn try_from(ref_kind: u8, ref_index: u16) -> ImageResult<Self> {
        let ref_idx = ref_index as usize;
        
        let res = match ref_kind {
            1 => GetField(ref_idx),
            2 => GetStatic(ref_idx),
            3 => PutField(ref_idx),
            4 => PutStatic(ref_idx),
            5 => InvokeVirtual(ref_idx),
            6 => InvokeStatic(ref_idx),
            7 => InvokeSpecial(ref_idx),
            8 => NewInvokeSpecial(ref_idx),
            9 => InvokeInterface(ref_idx),

            _ => return Err(ImageError::InvalidRefKind(ref_kind)),
        };

        Ok(res)
    }
}

#[derive(Debug)]
pub enum CPImageEntry<'a> {
    Utf8(&'a str),
    
    Class(&'a str),
    
    NameAndType(&'a str, &'a str),

    Fieldref {
        class_name: &'a str,
        name: &'a str,
        desc: FieldDescImage<'a>,
    },

    Methodref {
        class_name: &'a str,
        name: &'a str,
        desc: MethodDescImage<'a>,
    },

    InterfaceMethodref {
        class_name: &'a str,
        name: &'a str,
        desc: MethodDescImage<'a>,
    },

    String(&'a str),

    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),

    MethodHandle(MethodHandleImage),

    MethodType(&'a str),

    InvokeDynamic {
        bs_method_attr_idx: usize,
        name: &'a str,
        desc: MethodDescImage<'a>,
    },
}

pub struct CPImage<'a> {
    entries: Vec<OnceCell<CPImageEntry<'a>>>,
}

impl<'a> CPImage<'a> {
    pub(super) fn try_new(infos: &'a [ConstantPoolInfo]) -> ImageResult<Self> {
        let entries = (0..infos.len())
            .map(|_| OnceCell::new())
            .collect();

        let res = Self { entries };
        res.init(infos)?;

        Ok(res)
    }
    
    fn init(&self, infos: &'a [ConstantPoolInfo]) -> ImageResult<()> {
        for idx in 0..infos.len() {
            let entry = match &infos[idx] {
                ConstantPoolInfo::Class { .. } => {
                    get_class_name(&self.entries, infos, idx as u16)?;
                    None
                },

                ConstantPoolInfo::Double(n) => Some(CPImageEntry::Double(*n)),

                ConstantPoolInfo::Fieldref { class_index, name_and_type_index } => {
                    let class_name = get_class_name(&self.entries, infos, *class_index)?;
                    let (name, desc_raw) = get_name_and_type(&self.entries, infos, *name_and_type_index)?;
                    let desc = FieldDescImage::try_from(desc_raw)?;

                    Some(CPImageEntry::Fieldref { class_name, name, desc })
                },

                ConstantPoolInfo::Float(n) => Some(CPImageEntry::Float(*n)),

                ConstantPoolInfo::Integer(n) => Some(CPImageEntry::Integer(*n)),

                ConstantPoolInfo::InterfaceMethodref { class_index, name_and_type_index } => {
                    let class_name = get_class_name(&self.entries, infos, *class_index)?;
                    let (name, desc_raw) = get_name_and_type(&self.entries, infos, *name_and_type_index)?;
                    let desc = MethodDescImage::try_from(desc_raw)?;

                    Some(CPImageEntry::InterfaceMethodref { class_name, name, desc })
                },

                ConstantPoolInfo::InvokeDynamic { bs_method_attr_index, name_and_type_index } => {
                    let (name, desc_raw) = get_name_and_type(&self.entries, infos, *name_and_type_index)?;
                    let desc = MethodDescImage::try_from(desc_raw)?;
                    
                    let bs_method_attr_idx = *bs_method_attr_index as usize;
                    
                    Some(CPImageEntry::InvokeDynamic {
                        bs_method_attr_idx,
                        name,
                        desc,
                    })
                },

                ConstantPoolInfo::Long(n) => Some(CPImageEntry::Long(*n)),

                ConstantPoolInfo::MethodHandle { ref_kind, ref_index } => Some(CPImageEntry::MethodHandle(
                    MethodHandleImage::try_from(*ref_kind, *ref_index)?,
                )),

                ConstantPoolInfo::Methodref { class_index, name_and_type_index } => {
                    let class_name = get_class_name(&self.entries, infos, *class_index)?;
                    let (name, desc_raw) = get_name_and_type(&self.entries, infos, *name_and_type_index)?;
                    let desc = MethodDescImage::try_from(desc_raw)?;

                    Some(CPImageEntry::Methodref { class_name, name, desc })
                },

                ConstantPoolInfo::MethodType { desc_index } => Some(CPImageEntry::MethodType(
                    get_utf8(infos, *desc_index)?,
                )),

                ConstantPoolInfo::NameAndType { .. } => {
                    get_name_and_type(&self.entries, infos, idx as u16)?;
                    None
                },

                ConstantPoolInfo::String { string_index } => Some(CPImageEntry::String(
                    get_utf8(infos, *string_index)?,
                )),

                ConstantPoolInfo::Unusable => None,

                ConstantPoolInfo::Utf8(raw) => Some(CPImageEntry::Utf8(raw.as_str())),
            };

            if let Some(x) = entry {
                self.entries[idx].set(x).unwrap();
            }
        }

        Ok(())
    }
}

impl<'a> CPImage<'a> {
    pub fn get(
        &self,
        idx: u16,
    ) -> ImageResult<&CPImageEntry<'a>> {
        let cell = self
            .entries
            .get(idx as usize)
            .ok_or(ImageError::InvalidCPIndex(idx))?;

        cell.get()
            .ok_or(ImageError::InvalidCPIndex(idx))
    }
}

impl<'cf> CPImage<'cf> {
    pub fn utf8(&self, idx: u16) -> ImageResult<&'cf str> {
        match self.get(idx)? {
            CPImageEntry::Utf8(value) => Ok(*value),
            _ => Err(ImageError::CPKindMismatched {
                index: idx,
                expected: CPKind::Utf8,
            }),
        }
    }

    pub fn class(&self, idx: u16) -> ImageResult<&'cf str> {
        match self.get(idx)? {
            CPImageEntry::Class(name) => Ok(*name),
            _ => Err(ImageError::CPKindMismatched {
                index: idx,
                expected: CPKind::Class,
            }),
        }
    }
}
fn get_utf8(
    infos: &[ConstantPoolInfo],
    index: u16,
) -> ImageResult<&str> {
    let Some(info) = infos.get(index as usize) else {
        return Err(ImageError::InvalidCPIndex(index));
    };
    match info {
        ConstantPoolInfo::Utf8(raw) => Ok(raw.as_str()),

        _ => return Err(ImageError::CPKindMismatched {
            index,
            expected: CPKind::Utf8,
        })
    }
}

fn get_class_name<'a>(
    target: &[OnceCell<CPImageEntry<'a>>],
    infos: &'a [ConstantPoolInfo],
    index: u16,
) -> ImageResult<&'a str> {
    let Some(entry) = target.get(index as usize) else {
        return Err(ImageError::InvalidCPIndex(index));
    };

    if let Some(x) = entry.get() {
        match x {
            CPImageEntry::Class(name) => return Ok(name),
            _ => return Err(ImageError::CPKindMismatched {
                index,
                expected: CPKind::Class,
            })
        }
    }

    let Some(info) = infos.get(index as usize) else {
        return Err(ImageError::InvalidCPIndex(index));
    };
    match info {
        ConstantPoolInfo::Class { name_index } => {
            let name = get_utf8(infos, *name_index)?;
            entry.set(CPImageEntry::Class(name)).unwrap();
            Ok(name)
        }

        _ => return Err(ImageError::CPKindMismatched {
            index,
            expected: CPKind::Class,
        })
    }
}

fn get_name_and_type<'a>(
    target: &[OnceCell<CPImageEntry<'a>>],
    infos: &'a [ConstantPoolInfo],
    index: u16,
) -> ImageResult<(&'a str, &'a str)> {
    let Some(entry) = target.get(index as usize) else {
        return Err(ImageError::InvalidCPIndex(index));
    };

    if let Some(x) = entry.get() {
        match x {
            CPImageEntry::NameAndType(name, desc) => return Ok((name, desc)),
            _ => return Err(ImageError::CPKindMismatched {
                index,
                expected: CPKind::NameAndType,
            })
        }
    }

    let Some(info) = infos.get(index as usize) else {
        return Err(ImageError::InvalidCPIndex(index));
    };
    match info {
        ConstantPoolInfo::NameAndType { name_index, desc_index } => {
            let name = get_utf8(infos, *name_index)?;
            let desc = get_utf8(infos, *desc_index)?;
            entry.set(CPImageEntry::NameAndType(name, desc)).unwrap();

            Ok((name, desc))
        }

        _ => return Err(ImageError::CPKindMismatched {
            index,
            expected: CPKind::NameAndType,
        })
    }
}
