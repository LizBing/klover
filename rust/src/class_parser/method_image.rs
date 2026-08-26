use std::marker::PhantomData;

use crate::class_parser::{
    acc_flags::MethodAccFlags, attr_image::CodeImage, attr_info::AttrInfo, cp_image::CPImage,
    desc_image::MethodDescImage, method_info::MethodInfo, parse_error::ImageResult,
};

pub struct MethodImage<'a> {
    __: PhantomData<()>,

    pub name: &'a str,
    pub desc: MethodDescImage<'a>,
    pub acc_flags: MethodAccFlags,

    pub code: Option<CodeImage<'a>>,
}

impl<'a> MethodImage<'a> {
    pub(super) fn try_from(info: &'a MethodInfo, cp: &CPImage<'a>) -> ImageResult<Self> {
        let name = cp.utf8(info.name_idx)?;
        let desc_raw = cp.utf8(info.desc_idx)?;
        let desc = MethodDescImage::try_from(desc_raw)?;
        let acc_flags = MethodAccFlags::from_bits_retain(info.acc_flags);

        let mut code = None;
        for info in &info.attrs {
            match info {
                AttrInfo::Code(x) => code = Some(CodeImage::try_from(x, cp)?),

                _ => continue,
            }
        }

        Ok(Self {
            __: PhantomData,
            name,
            desc,
            acc_flags,
            code,
        })
    }
}
