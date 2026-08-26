use std::marker::PhantomData;

use crate::class_parser::{
    acc_flags::ClassAccFlags, class_file::ClassFile, cp_image::CPImage, field_image::FieldImage,
    method_image::MethodImage, parse_error::ImageResult,
};

pub struct ClassImage<'a> {
    __: PhantomData<()>,

    pub cp: CPImage<'a>,

    pub name: &'a str,
    pub super_name: Option<&'a str>,
    pub acc_flags: ClassAccFlags,
    pub interfaces: Vec<&'a str>,
    pub fields: Vec<FieldImage<'a>>,
    pub methods: Vec<MethodImage<'a>>,
}

impl<'a> ClassImage<'a> {
    pub fn new(cf: &'a ClassFile) -> ImageResult<Self> {
        let cp = CPImage::try_new(&cf.constant_pool)?;

        let name = cp.class(cf.this_class)?;
        let super_name = if cf.super_class == 0 {
            None
        } else {
            Some(cp.class(cf.super_class)?)
        };

        let acc_flags = ClassAccFlags::from_bits_retain(cf.acc_flags);

        let mut interfaces = Vec::new();
        for n in &cf.interfaces {
            let iface = cp.class(*n)?;
            interfaces.push(iface);
        }

        let mut fields = Vec::new();
        for n in &cf.fields {
            let field = FieldImage::try_from(n, &cp)?;
            fields.push(field);
        }

        let mut methods = Vec::new();
        for n in &cf.methods {
            let method = MethodImage::try_from(n, &cp)?;
            methods.push(method);
        }

        Ok(Self {
            __: PhantomData,
            cp,
            name,
            super_name,
            acc_flags,
            interfaces,
            fields,
            methods,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;
    use crate::class_parser::{acc_flags::MethodAccFlags, cp_image::CPImageEntry};

    fn fixture_path(relative_path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../test_data/classes")
            .join(relative_path)
    }

    fn read_class_file(relative_path: &str) -> ClassFile {
        let path = fixture_path(relative_path);
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

        ClassFile::from(&bytes)
            .unwrap_or_else(|error| panic!("failed to parse {}: {error:?}", path.display()))
    }

    #[test]
    fn builds_java_lang_object_image() {
        let class_file = read_class_file("java/lang/Object.class");
        let image = ClassImage::new(&class_file).unwrap();

        assert_eq!(image.name, "java/lang/Object");
        assert_eq!(image.super_name, None);
        assert!(image.interfaces.is_empty());
        assert!(image.fields.is_empty());

        let constructor = image
            .methods
            .iter()
            .find(|method| method.name == "<init>" && method.desc.raw == "()V")
            .expect("java/lang/Object must declare <init>:()V");
        let code = constructor
            .code
            .as_ref()
            .expect("constructor must have Code");

        assert_eq!(code.code.as_slice(), [0xb1]); // return
    }

    #[test]
    fn builds_simple_addition_image() {
        let class_file = read_class_file("SimpleAddition.class");
        let image = ClassImage::new(&class_file).unwrap();

        assert_eq!(image.name, "SimpleAddition");
        assert_eq!(image.super_name, Some("java/lang/Object"));

        let add = image
            .methods
            .iter()
            .find(|method| method.name == "add" && method.desc.raw == "(II)I")
            .expect("SimpleAddition must declare add:(II)I");

        assert!(add.acc_flags.contains(MethodAccFlags::ACC_PUBLIC));
        assert!(add.acc_flags.contains(MethodAccFlags::ACC_STATIC));

        let code = add.code.as_ref().expect("add must have Code");
        assert_eq!(code.max_stack, 2);
        assert_eq!(code.max_locals, 2);
        assert_eq!(code.code.as_slice(), [0x1a, 0x1b, 0x60, 0xac]);
    }

    #[test]
    fn builds_hello_klover_image_and_resolves_required_cp_entries() {
        let class_file = read_class_file("HelloKlover.class");
        let image = ClassImage::new(&class_file).unwrap();

        assert_eq!(image.name, "HelloKlover");
        assert_eq!(image.super_name, Some("java/lang/Object"));

        match image.cp.get(7).unwrap() {
            CPImageEntry::Fieldref {
                class_name,
                name,
                desc,
            } => {
                assert_eq!(*class_name, "java/lang/System");
                assert_eq!(*name, "out");
                assert_eq!(desc.raw, "Ljava/io/PrintStream;");
            }
            entry => panic!("CP #7 must be Fieldref, got {entry:?}"),
        }

        match image.cp.get(13).unwrap() {
            CPImageEntry::String(value) => assert_eq!(*value, "Hello Klover!"),
            entry => panic!("CP #13 must be String, got {entry:?}"),
        }

        match image.cp.get(15).unwrap() {
            CPImageEntry::Methodref {
                class_name,
                name,
                desc,
            } => {
                assert_eq!(*class_name, "java/io/PrintStream");
                assert_eq!(*name, "println");
                assert_eq!(desc.raw, "(Ljava/lang/String;)V");
            }
            entry => panic!("CP #15 must be Methodref, got {entry:?}"),
        }

        let main = image
            .methods
            .iter()
            .find(|method| method.name == "main" && method.desc.raw == "([Ljava/lang/String;)V")
            .expect("HelloKlover must declare main:([Ljava/lang/String;)V");

        assert!(main.acc_flags.contains(MethodAccFlags::ACC_PUBLIC));
        assert!(main.acc_flags.contains(MethodAccFlags::ACC_STATIC));

        let code = main.code.as_ref().expect("main must have Code");
        assert_eq!(code.max_stack, 2);
        assert_eq!(code.max_locals, 1);
        assert_eq!(
            code.code.as_slice(),
            [0xb2, 0x00, 0x07, 0x12, 0x0d, 0xb6, 0x00, 0x0f, 0xb1]
        );
    }
}
