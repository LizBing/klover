use std::marker::PhantomData;

use crate::class_parser::{
    acc_flags::FieldAccFlags, attr_image::ConstantValueImage, attr_info::AttrInfo,
    cp_image::CPImage, desc_image::FieldDescImage, field_info::FieldInfo, parse_error::ImageResult,
};

pub struct FieldImage<'a> {
    __: PhantomData<()>,

    pub name: &'a str,
    pub desc: FieldDescImage<'a>,
    pub acc_flags: FieldAccFlags,

    pub constant_value: Option<ConstantValueImage>,
}

impl<'a> FieldImage<'a> {
    pub(super) fn try_from(info: &FieldInfo, cp: &CPImage<'a>) -> ImageResult<Self> {
        let name = cp.utf8(info.name_idx)?;
        let desc_raw = cp.utf8(info.desc_idx)?;
        let desc = FieldDescImage::try_from(desc_raw)?;
        let acc_flags = FieldAccFlags::from_bits_retain(info.acc_flags);

        let mut constant_value = None;
        for info in &info.attrs {
            match info {
                AttrInfo::ConstantValue { cp_idx } => {
                    constant_value = Some(ConstantValueImage::try_from(cp, *cp_idx)?)
                }

                _ => continue,
            }
        }

        Ok(Self {
            __: PhantomData,
            name,
            desc,
            acc_flags,
            constant_value,
        })
    }
}
