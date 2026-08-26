use std::marker::PhantomData;

use crate::class_parser::parse_error::ImageError;

#[derive(Debug)]
pub enum ElemDescImage<'a> {
    Boolean,
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Class(&'a str),
}

#[derive(Debug)]
pub struct FieldDescImage<'a> {
    __: PhantomData<()>,

    pub raw: &'a str,
    pub dimensions: usize,
    pub elem: ElemDescImage<'a>,
}

impl<'a> FieldDescImage<'a> {
    /// Parses a field descriptor at the start of `value`, returning the
    /// parsed image together with the number of bytes it consumed.
    fn parse_prefix(value: &'a str) -> Result<(Self, usize), ImageError> {
        let invalid = || ImageError::InvalidDesc(value.to_string());
        let bytes = value.as_bytes();
        let mut pos = 0;

        // Parse array dimensions.
        while pos < bytes.len() && bytes[pos] == b'[' {
            pos += 1;
        }
        let dimensions = pos;

        if pos >= bytes.len() {
            return Err(invalid());
        }

        let elem = match bytes[pos] {
            b'B' => ElemDescImage::Byte,
            b'C' => ElemDescImage::Char,
            b'D' => ElemDescImage::Double,
            b'F' => ElemDescImage::Float,
            b'I' => ElemDescImage::Int,
            b'J' => ElemDescImage::Long,
            b'S' => ElemDescImage::Short,
            b'Z' => ElemDescImage::Boolean,
            b'L' => {
                // Class type: L<classname>;
                let start = pos + 1;
                match bytes[start..].iter().position(|&b| b == b';') {
                    // The class name must be non-empty.
                    Some(rel) if rel > 0 => {
                        pos = start + rel; // `pos` now points at ';'
                        ElemDescImage::Class(&value[start..pos])
                    }
                    _ => return Err(invalid()),
                }
            }
            _ => return Err(invalid()),
        };

        let consumed = pos + 1;
        let image = FieldDescImage {
            __: PhantomData,

            raw: &value[..consumed],
            dimensions,
            elem,
        };

        Ok((image, consumed))
    }
}

impl<'a> TryFrom<&'a str> for FieldDescImage<'a> {
    type Error = ImageError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (image, consumed) = Self::parse_prefix(value)?;

        // A field descriptor must be consumed in full.
        if consumed != value.len() {
            return Err(ImageError::InvalidDesc(value.to_string()));
        }

        Ok(image)
    }
}

#[derive(Debug)]
pub enum ReturnDescImage<'a> {
    Void,
    Type(FieldDescImage<'a>),
}

#[derive(Debug)]
pub struct MethodDescImage<'a> {
    __: PhantomData<()>,

    pub raw: &'a str,
    pub ret: ReturnDescImage<'a>,
    pub args: Vec<FieldDescImage<'a>>,
}

impl<'a> TryFrom<&'a str> for MethodDescImage<'a> {
    type Error = ImageError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let invalid = || ImageError::InvalidDesc(value.to_string());
        let bytes = value.as_bytes();

        if bytes.first() != Some(&b'(') {
            return Err(invalid());
        }

        // The first ')' terminates the parameter list.
        let close_paren = match bytes[1..].iter().position(|&b| b == b')') {
            Some(rel) => rel + 1,
            None => return Err(invalid()),
        };

        // Parse parameter descriptors - each is a complete field descriptor.
        let mut args = Vec::new();
        let mut pos = 1; // right after '('
        while pos < close_paren {
            let (arg, consumed) =
                FieldDescImage::parse_prefix(&value[pos..close_paren]).map_err(|_| invalid())?;
            args.push(arg);
            pos += consumed;
        }

        // Parse the return descriptor.
        let ret_str = &value[close_paren + 1..];
        let ret = match ret_str.as_bytes() {
            [b'V'] => ReturnDescImage::Void,
            // 'V' must stand alone, and the return type must not be empty.
            [b'V', ..] | [] => return Err(invalid()),
            _ => ReturnDescImage::Type(FieldDescImage::try_from(ret_str).map_err(|_| invalid())?),
        };

        Ok(MethodDescImage {
            __: PhantomData,

            raw: value,
            ret,
            args,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_primitive_field_desc() {
        let image = FieldDescImage::try_from("I").unwrap();
        assert_eq!(image.raw, "I");
        assert_eq!(image.dimensions, 0);
        assert!(matches!(image.elem, ElemDescImage::Int));
    }

    #[test]
    fn parses_array_of_class_field_desc() {
        let image = FieldDescImage::try_from("[[Ljava/lang/String;").unwrap();
        assert_eq!(image.raw, "[[Ljava/lang/String;");
        assert_eq!(image.dimensions, 2);
        assert!(matches!(
            image.elem,
            ElemDescImage::Class("java/lang/String")
        ));
    }

    #[test]
    fn rejects_invalid_field_descs() {
        for desc in ["", "V", "[", "X", "II", "Ljava/lang/String", "L;", "[;"] {
            assert!(
                FieldDescImage::try_from(desc).is_err(),
                "expected error for {desc:?}"
            );
        }
    }

    #[test]
    fn parses_method_desc() {
        let image = MethodDescImage::try_from("(IJLjava/lang/Object;[D)V").unwrap();
        assert_eq!(image.raw, "(IJLjava/lang/Object;[D)V");
        assert!(matches!(image.ret, ReturnDescImage::Void));

        assert_eq!(image.args.len(), 4);
        assert!(matches!(image.args[0].elem, ElemDescImage::Int));
        assert!(matches!(image.args[1].elem, ElemDescImage::Long));
        assert!(matches!(
            image.args[2].elem,
            ElemDescImage::Class("java/lang/Object")
        ));
        assert_eq!(image.args[3].raw, "[D");
        assert_eq!(image.args[3].dimensions, 1);
        assert!(matches!(image.args[3].elem, ElemDescImage::Double));
    }

    #[test]
    fn parses_non_void_return() {
        let image = MethodDescImage::try_from("()[[I").unwrap();
        assert!(image.args.is_empty());
        match image.ret {
            ReturnDescImage::Type(ret) => {
                assert_eq!(ret.dimensions, 2);
                assert!(matches!(ret.elem, ElemDescImage::Int));
            }
            _ => panic!("expected non-void return"),
        }
    }

    #[test]
    fn rejects_invalid_method_descs() {
        for desc in [
            "",
            "()",
            "(I",
            "I)V",
            "(V)V",
            "(I)VLjava/lang/String;",
            "((I)V",
            "(Lfoo)V",
        ] {
            assert!(
                MethodDescImage::try_from(desc).is_err(),
                "expected error for {desc:?}"
            );
        }
    }
}
