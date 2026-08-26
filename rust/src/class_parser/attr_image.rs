use std::marker::PhantomData;

use crate::{class_parser::{attr_info::{CodeAttrInfo, ExceptionTableEntryInfo}, cp_image::{CPImage, CPImageEntry}, parse_error::{CPKind, ImageError, ImageResult}},};

#[derive(Debug)]
pub enum ConstantValueImage {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

impl ConstantValueImage {
    pub(super) fn try_from(cp: &CPImage, idx: u16) -> ImageResult<Self> {
        let res = match cp.get(idx)? {
            CPImageEntry::Integer(n) => Self::Int(*n),
            CPImageEntry::Long(n) => Self::Long(*n),
            CPImageEntry::Float(n) => Self::Float(*n),
            CPImageEntry::Double(n) => Self::Double(*n),
            _ => return Err(ImageError::CPKindMismatched { index: idx, expected: CPKind::ConstantValue }),
        };

        Ok(res)
    }
}

#[derive(Debug)]
pub struct ExceptionTableEntryImage<'a> {
    __: PhantomData<()>,
    
    pub start_pc: usize,
    pub end_pc: usize,
    pub handler_pc: usize,
    pub catch_type: Option<&'a str>,
}

impl<'a> ExceptionTableEntryImage<'a> {
    pub fn try_from(info: &ExceptionTableEntryInfo, cp: &CPImage<'a>) -> ImageResult<Self> {
        let catch_type = if info.catch_type == 0 {
            None
        } else {
            Some(cp.class(info.catch_type)?)
        };

        Ok(Self {
            __: PhantomData,
            start_pc: info.start_pc as usize,
            end_pc: info.end_pc as usize,
            handler_pc: info.handler_pc as usize,
            catch_type
        })
    }
}

#[derive(Debug)]
pub struct CodeImage<'a> {
    __: PhantomData<()>,
    
    pub max_stack: usize,
    pub max_locals: usize,
    pub code: &'a Vec<u8>,
    pub exception_table: Vec<ExceptionTableEntryImage<'a>>,
    // pub attrs: Vec<AttrImage<'a>>,
}

impl<'a> CodeImage<'a> {
    pub fn try_from(info: &'a CodeAttrInfo, cp: &CPImage<'a>) -> ImageResult<Self> {
        let mut exception_table = Vec::with_capacity(info.exception_table.len());
        for entry in &info.exception_table {
            exception_table.push(ExceptionTableEntryImage::try_from(entry, cp)?);
        }

        // let mut attrs = Vec::with_capacity(info.attrs.len());
        // for n in &info.attrs {
        //     attrs.push(AttrImage::try_from(n, cp)?);
        // }

        Ok(Self {
            __: PhantomData,

            max_stack: info.max_stack as usize,
            max_locals: info.max_locals as usize,
            code: &info.code,
            exception_table,
            // attrs
        })
    }
}
