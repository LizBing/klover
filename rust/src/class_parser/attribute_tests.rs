//! Small class-file fixtures exercise attribute contents and cross-consumer identity.
use super::attributes::*;
use super::constant_pool::{ClassRef, LoadableConstant};
use super::{ClassImage, Constant, JavaText, Opcode, parse_class};
use std::sync::Arc;

#[derive(Clone)]
struct Attr {
    name: u16,
    body: Vec<u8>,
}
struct Member {
    flags: u16,
    name: u16,
    descriptor: u16,
    attrs: Vec<Attr>,
}
struct Fixture {
    cp: Vec<Vec<u8>>,
    this: u16,
    parent: u16,
    fields: Vec<Member>,
    methods: Vec<Member>,
    attrs: Vec<Attr>,
}
fn u2(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend(value.to_be_bytes());
}
fn words(values: &[u16]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}
fn write_attrs(bytes: &mut Vec<u8>, attrs: &[Attr]) {
    u2(bytes, attrs.len() as u16);
    for attr in attrs {
        u2(bytes, attr.name);
        bytes.extend((attr.body.len() as u32).to_be_bytes());
        bytes.extend(&attr.body);
    }
}
impl Fixture {
    fn new() -> Self {
        let mut f = Self {
            cp: Vec::new(),
            this: 0,
            parent: 0,
            fields: Vec::new(),
            methods: Vec::new(),
            attrs: Vec::new(),
        };
        f.this = f.class("Test");
        f.parent = f.class("java/lang/Object");
        f
    }
    fn entry(&mut self, bytes: Vec<u8>) -> u16 {
        self.cp.push(bytes);
        self.cp.len() as u16
    }
    fn utf8(&mut self, s: &str) -> u16 {
        self.java_text(JavaText::from_str(s))
    }
    fn java_text(&mut self, text: JavaText) -> u16 {
        let encoded = text.to_modified_utf8();
        let mut bytes = vec![1];
        u2(&mut bytes, encoded.len() as u16);
        bytes.extend(encoded);
        self.entry(bytes)
    }
    fn indexed(&mut self, tag: u8, index: u16) -> u16 {
        let mut bytes = vec![tag];
        u2(&mut bytes, index);
        self.entry(bytes)
    }
    fn class(&mut self, s: &str) -> u16 {
        let name = self.utf8(s);
        self.indexed(7, name)
    }
    fn integer(&mut self, value: i32) -> u16 {
        let mut bytes = vec![3];
        bytes.extend(value.to_be_bytes());
        self.entry(bytes)
    }
    fn numeric(&mut self, tag: u8, bits: u64) -> u16 {
        let mut bytes = vec![tag];
        if tag == 4 {
            bytes.extend((bits as u32).to_be_bytes());
        } else {
            bytes.extend(bits.to_be_bytes());
        }
        let index = self.entry(bytes);
        if matches!(tag, 5 | 6) {
            self.cp.push(Vec::new());
        }
        index
    }
    fn nat(&mut self, name: &str, desc: &str) -> u16 {
        let name = self.utf8(name);
        let desc = self.utf8(desc);
        let mut bytes = vec![12];
        bytes.extend(words(&[name, desc]));
        self.entry(bytes)
    }
    fn method_ref(&mut self) -> u16 {
        let nat = self.nat("bootstrap", "()V");
        let mut bytes = vec![10];
        bytes.extend(words(&[self.parent, nat]));
        self.entry(bytes)
    }
    fn handle(&mut self, target: u16) -> u16 {
        let mut bytes = vec![15, 6];
        u2(&mut bytes, target);
        self.entry(bytes)
    }
    fn attr(&mut self, name: &str, body: Vec<u8>) -> Attr {
        Attr {
            name: self.utf8(name),
            body,
        }
    }
    fn class_attr(&mut self, name: &str, body: Vec<u8>) {
        let attr = self.attr(name, body);
        self.attrs.push(attr);
    }
    fn field(&mut self, name: &str, desc: &str, attrs: Vec<Attr>) {
        let name = self.utf8(name);
        let descriptor = self.utf8(desc);
        self.fields.push(Member {
            flags: 0x19,
            name,
            descriptor,
            attrs,
        });
    }
    fn method(&mut self, name: &str, desc: &str, flags: u16, attrs: Vec<Attr>) {
        let name = self.utf8(name);
        let descriptor = self.utf8(desc);
        self.methods.push(Member {
            flags,
            name,
            descriptor,
            attrs,
        });
    }
    fn code(
        &mut self,
        bytes: &[u8],
        max_stack: u16,
        max_locals: u16,
        handlers: &[u16],
        nested: Vec<Attr>,
    ) -> Attr {
        let mut body = words(&[max_stack, max_locals]);
        body.extend((bytes.len() as u32).to_be_bytes());
        body.extend(bytes);
        u2(&mut body, (handlers.len() / 4) as u16);
        body.extend(words(handlers));
        write_attrs(&mut body, &nested);
        self.attr("Code", body)
    }
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 52];
        u2(&mut bytes, self.cp.len() as u16 + 1);
        for entry in &self.cp {
            bytes.extend(entry);
        }
        bytes.extend(words(&[0x21, self.this, self.parent, 0]));
        for members in [&self.fields, &self.methods] {
            u2(&mut bytes, members.len() as u16);
            for member in members {
                bytes.extend(words(&[member.flags, member.name, member.descriptor]));
                write_attrs(&mut bytes, &member.attrs);
            }
        }
        write_attrs(&mut bytes, &self.attrs);
        bytes
    }
    fn parse(&self) -> ClassImage {
        parse_class(&self.bytes()).unwrap()
    }
}
fn code(image: &ClassImage, method: usize) -> &super::CodeData {
    image.methods()[method].attributes.code.as_ref().unwrap()
}
fn ldc_class(opcode: &Opcode) -> &Arc<ClassRef> {
    let Opcode::Ldc(value) = opcode else {
        panic!("expected ldc")
    };
    let LoadableConstant::Class(class) = value.as_ref() else {
        panic!("expected class")
    };
    class
}

#[test]
fn attributes_and_instructions_share_canonical_constants_across_methods() {
    let mut f = Fixture::new();
    let number = f.integer(42);
    let duplicate = f.integer(42);
    let value = f.attr("ConstantValue", words(&[number]));
    f.field("answer", "I", vec![value]);
    let member = f.method_ref();
    let handle = f.handle(member);
    let mut bootstrap = words(&[1, handle, 3, number, f.parent, handle]);
    f.class_attr("BootstrapMethods", std::mem::take(&mut bootstrap));
    let nat = f.nat("call", "()V");
    let mut entry = vec![18];
    entry.extend(words(&[0, nat]));
    let dynamic = f.entry(entry);
    // full_frame at offset 0: one Object local, empty operand stack.
    let mut frame = words(&[1]);
    frame.push(255);
    frame.extend(words(&[0, 1]));
    frame.push(7);
    frame.extend(words(&[f.parent, 0]));
    let stack_map = f.attr("StackMapTable", frame);
    let exceptions = f.attr("Exceptions", words(&[1, f.parent]));
    let mut bytes = vec![
        0x12,
        number as u8,
        0x12,
        duplicate as u8,
        0x12,
        f.parent as u8,
        0xbb,
    ];
    bytes.extend(words(&[f.parent]));
    bytes.push(0xb8);
    bytes.extend(words(&[member]));
    bytes.extend([0x12, handle as u8, 0xba]);
    bytes.extend(words(&[dynamic, 0]));
    bytes.push(0xb1);
    let body = f.code(
        &bytes,
        8,
        1,
        &[
            0,
            bytes.len() as u16,
            0,
            f.parent,
            0,
            bytes.len() as u16,
            0,
            0,
        ],
        vec![stack_map],
    );
    f.method("first", "()V", 9, vec![body, exceptions]);
    let body = f.code(&[0x12, number as u8, 0xb1], 1, 0, &[], vec![]);
    f.method("second", "()V", 9, vec![body]);
    let image = f.parse();
    let opcodes = &code(&image, 0).bytecode.opcodes;
    let Opcode::Ldc(first) = &opcodes[0].1 else {
        panic!()
    };
    let Opcode::Ldc(equal_but_distinct) = &opcodes[1].1 else {
        panic!()
    };
    let Opcode::Ldc(second_method) = &code(&image, 1).bytecode.opcodes[0].1 else {
        panic!()
    };
    assert!(Arc::ptr_eq(first, second_method));
    assert!(Arc::ptr_eq(
        first,
        image.fields()[0]
            .attributes
            .constant_value
            .as_ref()
            .unwrap()
    ));
    assert!(!Arc::ptr_eq(first, equal_but_distinct));
    let bootstrap = &image.attributes().bootstrap_methods.as_ref().unwrap()[0];
    assert!(Arc::ptr_eq(first, &bootstrap.arguments[0]));
    let class = ldc_class(&opcodes[2].1);
    assert!(Arc::ptr_eq(class, image.super_class().unwrap()));
    assert_eq!(image.this_class().name, "Test");
    let Opcode::New(new_class) = &opcodes[3].1 else {
        panic!()
    };
    assert!(Arc::ptr_eq(class, new_class));
    assert!(Arc::ptr_eq(
        class,
        code(&image, 0).exception_table[0]
            .catch_type
            .as_ref()
            .unwrap()
    ));
    assert!(code(&image, 0).exception_table[1].catch_type.is_none());
    assert!(Arc::ptr_eq(
        class,
        &image.methods()[0].attributes.exceptions.as_ref().unwrap()[0]
    ));
    let StackMapFrameKind::Full { locals, .. } =
        &code(&image, 0).attributes.stack_map_table.as_ref().unwrap()[0].kind
    else {
        panic!()
    };
    let VerificationType::Object(stack_class) = &locals[0] else {
        panic!()
    };
    assert!(Arc::ptr_eq(class, stack_class));
    let LoadableConstant::Class(bootstrap_class) = bootstrap.arguments[1].as_ref() else {
        panic!()
    };
    assert!(Arc::ptr_eq(class, bootstrap_class));
    let Opcode::InvokeStatic(method) = &opcodes[4].1 else {
        panic!()
    };
    assert!(Arc::ptr_eq(method, &bootstrap.method.target));
    assert!(Arc::ptr_eq(class, &method.owner));
    let Opcode::Ldc(loaded_handle) = &opcodes[5].1 else {
        panic!()
    };
    let LoadableConstant::MethodHandle(loaded_handle) = loaded_handle.as_ref() else {
        panic!()
    };
    assert!(Arc::ptr_eq(loaded_handle, &bootstrap.method));
    let LoadableConstant::MethodHandle(argument_handle) = bootstrap.arguments[2].as_ref() else {
        panic!()
    };
    assert!(Arc::ptr_eq(loaded_handle, argument_handle));
    let Opcode::InvokeDynamic(dynamic) = &opcodes[6].1 else {
        panic!()
    };
    assert!(Arc::ptr_eq(&dynamic.bootstrap, bootstrap));
}

#[test]
fn exposes_class_field_and_method_metadata_and_preserves_unknown_bodies() {
    let mut f = Fixture::new();
    let source = f.utf8("Test.java");
    f.class_attr("SourceFile", words(&[source]));
    f.class_attr(
        "SourceDebugExtension",
        b"SMAP\nTest.java\nJava\n*S Java\n*E\n".to_vec(),
    );
    let signature = f.utf8("<T:Ljava/lang/Object;>Ljava/lang/Object;");
    f.class_attr("Signature", words(&[signature]));
    f.class_attr("Synthetic", vec![]);
    f.class_attr("Deprecated", vec![]);
    let inner_name = f.utf8("Inner");
    f.class_attr(
        "InnerClasses",
        words(&[2, f.this, f.parent, inner_name, 0x19, f.parent, 0, 0, 0]),
    );
    let enclosing = f.nat("outer", "()V");
    f.class_attr("EnclosingMethod", words(&[f.parent, enclosing]));
    let unknown_name = f.java_text(JavaText::from_utf16(vec![0xd800]));
    f.attrs.push(Attr {
        name: unknown_name,
        body: words(&[source]),
    });
    f.attrs.push(Attr {
        name: unknown_name,
        body: vec![8, 9, 10],
    });
    let literal = f.utf8("hello");
    let string = f.indexed(8, literal);
    let constant = f.attr("ConstantValue", words(&[string]));
    let sig = f.utf8("Ljava/lang/String;");
    let sig_attr = f.attr("Signature", words(&[sig]));
    let field_unknown = f.attr("VendorField", vec![1, 2]);
    f.field(
        "text",
        "Ljava/lang/String;",
        vec![constant, sig_attr, field_unknown],
    );
    let parameter = f.utf8("input");
    let mut parameters = vec![1];
    parameters.extend(words(&[parameter, 0x10]));
    let parameters = f.attr("MethodParameters", parameters);
    let exceptions = f.attr("Exceptions", words(&[0]));
    let method_unknown = f.attr("VendorMethod", vec![3]);
    f.method(
        "nativeMethod",
        "(I)V",
        0x109,
        vec![parameters, exceptions, method_unknown],
    );
    let image = f.parse();
    let attrs = image.attributes();
    assert!(attrs.synthetic && attrs.deprecated);
    assert!(attrs.source_file.as_ref().unwrap().eq_str("Test.java"));
    assert_eq!(
        &**attrs.source_debug_extension.as_ref().unwrap(),
        b"SMAP\nTest.java\nJava\n*S Java\n*E\n"
    );
    assert!(
        attrs
            .signature
            .as_ref()
            .unwrap()
            .eq_str("<T:Ljava/lang/Object;>Ljava/lang/Object;")
    );
    let inner = &attrs.inner_classes.as_ref().unwrap()[0];
    assert_eq!(inner.inner.name, "Test");
    assert_eq!(inner.outer.as_ref().unwrap().name, "java/lang/Object");
    assert!(inner.inner_name.as_ref().unwrap().eq_str("Inner"));
    assert_eq!(inner.access_flags, 0x19);
    assert!(attrs.inner_classes.as_ref().unwrap()[1].outer.is_none());
    assert!(
        attrs.inner_classes.as_ref().unwrap()[1]
            .inner_name
            .is_none()
    );
    let enclosing = attrs.enclosing_method.as_ref().unwrap();
    assert_eq!(enclosing.class.name, "java/lang/Object");
    let method = enclosing.method.as_ref().unwrap();
    assert!(method.name.eq_str("outer"));
    assert_eq!(method.descriptor.raw(), "()V");
    assert_eq!(attrs.unknown.len(), 2);
    assert_eq!(attrs.unknown[0].name.as_utf16(), [0xd800]);
    assert_eq!(&*attrs.unknown[0].body, words(&[source]));
    assert_eq!(&*attrs.unknown[1].body, [8, 9, 10]);
    let index = u16::from_be_bytes(attrs.unknown[0].body[..2].try_into().unwrap());
    assert!(
        matches!(image.constant(index), Some(Constant::Utf8(text)) if text.eq_str("Test.java"))
    );
    assert!(image.constant(0).is_none());
    assert!(image.constant(u16::MAX).is_none());
    let field = &image.fields()[0].attributes;
    assert!(
        matches!(field.constant_value.as_deref(), Some(LoadableConstant::String(value)) if value.eq_str("hello"))
    );
    assert!(
        field
            .signature
            .as_ref()
            .unwrap()
            .eq_str("Ljava/lang/String;")
    );
    assert_eq!(&*field.unknown[0].body, [1, 2]);
    let method = &image.methods()[0].attributes;
    assert!(method.code.is_none());
    assert!(method.exceptions.as_ref().unwrap().is_empty());
    assert!(
        method.parameters.as_ref().unwrap()[0]
            .name
            .as_ref()
            .unwrap()
            .eq_str("input")
    );
    assert_eq!(method.parameters.as_ref().unwrap()[0].access_flags, 0x10);
    assert_eq!(&*method.unknown[0].body, [3]);
}

#[test]
fn preserves_all_numeric_constant_kinds_and_unusable_slots() {
    let mut f = Fixture::new();
    let entries = [
        ("i", "I", f.integer(-3)),
        ("f", "F", f.numeric(4, 0x80000000)),
        ("l", "J", f.numeric(5, u64::MAX)),
        ("d", "D", f.numeric(6, 0x7ff8000000000001)),
    ];
    for (name, desc, index) in entries {
        let value = f.attr("ConstantValue", words(&[index]));
        f.field(name, desc, vec![value]);
    }
    let image = f.parse();
    assert!(matches!(
        image.fields()[0].attributes.constant_value.as_deref(),
        Some(LoadableConstant::Integer(-3))
    ));
    assert!(
        matches!(image.fields()[1].attributes.constant_value.as_deref(), Some(LoadableConstant::Float(value)) if value.to_bits() == 0x80000000)
    );
    assert!(matches!(
        image.fields()[2].attributes.constant_value.as_deref(),
        Some(LoadableConstant::Long(-1))
    ));
    assert!(
        matches!(image.fields()[3].attributes.constant_value.as_deref(), Some(LoadableConstant::Double(value)) if value.to_bits() == 0x7ff8000000000001)
    );
    assert!(image.constant(entries[2].2 + 1).is_none());
    assert!(image.constant(entries[3].2 + 1).is_none());
}

fn annotation_bytes(f: &mut Fixture, elements: Vec<(&str, Vec<u8>)>) -> Vec<u8> {
    let annotation_type = f.utf8("LExample;");
    let mut bytes = words(&[annotation_type, elements.len() as u16]);
    for (name, value) in elements {
        let name = f.utf8(name);
        u2(&mut bytes, name);
        bytes.extend(value);
    }
    bytes
}
fn counted(bytes: &[u8]) -> Vec<u8> {
    let mut out = words(&[1]);
    out.extend(bytes);
    out
}
fn tagged(tag: u8, index: u16) -> Vec<u8> {
    let mut out = vec![tag];
    u2(&mut out, index);
    out
}

#[test]
fn exposes_annotation_values_visibility_defaults_and_parameter_annotations() {
    let mut f = Fixture::new();
    let i = f.integer(7);
    let d = f.numeric(6, 3.0f64.to_bits());
    let l = f.numeric(5, 99);
    let fl = f.numeric(4, 2.0f32.to_bits() as u64);
    let text = f.java_text(JavaText::from_utf16(vec![0x61, 0, 0xd800]));
    let enum_type = f.utf8("LEnum;");
    let enum_name = f.utf8("VALUE");
    let class = f.utf8("[[Ljava/lang/Object;");
    let void = f.utf8("V");
    let nested = annotation_bytes(&mut f, vec![]);
    let mut nested_value = vec![b'@'];
    nested_value.extend(&nested);
    let mut enum_value = tagged(b'e', enum_type);
    u2(&mut enum_value, enum_name);
    let mut array = vec![b'['];
    u2(&mut array, 2);
    array.extend(tagged(b'I', i));
    array.extend(tagged(b's', text));
    let bytes = annotation_bytes(
        &mut f,
        vec![
            ("b", tagged(b'B', i)),
            ("c", tagged(b'C', i)),
            ("d", tagged(b'D', d)),
            ("f", tagged(b'F', fl)),
            ("i", tagged(b'I', i)),
            ("j", tagged(b'J', l)),
            ("s", tagged(b'S', i)),
            ("z", tagged(b'Z', i)),
            ("text", tagged(b's', text)),
            ("enum", enum_value),
            ("class", tagged(b'c', class)),
            ("void", tagged(b'c', void)),
            ("nested", nested_value),
            ("array", array),
        ],
    );
    f.class_attr("RuntimeVisibleAnnotations", counted(&bytes));
    f.class_attr("RuntimeInvisibleAnnotations", counted(&nested));
    let visible = f.attr("RuntimeVisibleAnnotations", counted(&bytes));
    let invisible = f.attr("RuntimeInvisibleAnnotations", counted(&nested));
    f.field("annotated", "I", vec![visible, invisible]);
    let visible = f.attr("RuntimeVisibleAnnotations", counted(&bytes));
    let invisible = f.attr("RuntimeInvisibleAnnotations", counted(&nested));
    let mut params = vec![1];
    params.extend(counted(&nested));
    let visible_params = f.attr("RuntimeVisibleParameterAnnotations", params.clone());
    let invisible_params = f.attr("RuntimeInvisibleParameterAnnotations", params);
    f.method(
        "annotated",
        "(I)V",
        0x109,
        vec![visible, invisible, visible_params, invisible_params],
    );
    let default = f.attr("AnnotationDefault", tagged(b's', text));
    f.method("defaultValue", "()Ljava/lang/String;", 0x401, vec![default]);
    let image = f.parse();
    let annotations = &image.attributes().annotations;
    assert_eq!(annotations.visible.as_ref().unwrap().len(), 1);
    assert_eq!(annotations.invisible.as_ref().unwrap().len(), 1);
    let annotation = &annotations.visible.as_ref().unwrap()[0];
    assert_eq!(annotation.annotation_type.raw(), "LExample;");
    assert_eq!(annotation.elements.len(), 14);
    for (index, tag) in [b'B', b'C', b'D', b'F', b'I', b'J', b'S', b'Z']
        .iter()
        .enumerate()
    {
        let ElementValue::Constant { tag: actual, .. } = &annotation.elements[index].1 else {
            panic!()
        };
        assert_eq!(actual, tag);
    }
    assert!(
        matches!(&annotation.elements[8].1, ElementValue::String(text) if text.as_utf16() == [0x61, 0, 0xd800])
    );
    assert!(
        matches!(&annotation.elements[9].1, ElementValue::Enum { enum_type, name } if enum_type.raw() == "LEnum;" && name.eq_str("VALUE"))
    );
    assert!(
        matches!(&annotation.elements[10].1, ElementValue::Class(Some(desc)) if desc.raw() == "[[Ljava/lang/Object;")
    );
    assert!(matches!(
        &annotation.elements[11].1,
        ElementValue::Class(None)
    ));
    assert!(
        matches!(&annotation.elements[12].1, ElementValue::Annotation(a) if a.annotation_type.raw() == "LExample;")
    );
    assert!(matches!(&annotation.elements[13].1, ElementValue::Array(values) if values.len() == 2));
    let class_value = match &annotation.elements[4].1 {
        ElementValue::Constant { value, .. } => value,
        _ => panic!(),
    };
    let field_value = match &image.fields()[0]
        .attributes
        .annotations
        .visible
        .as_ref()
        .unwrap()[0]
        .elements[4]
        .1
    {
        ElementValue::Constant { value, .. } => value,
        _ => panic!(),
    };
    assert!(Arc::ptr_eq(class_value, field_value));
    assert!(image.fields()[0].attributes.annotations.invisible.is_some());
    let method = &image.methods()[0].attributes;
    assert!(method.annotations.visible.is_some() && method.annotations.invisible.is_some());
    assert_eq!(
        method.visible_parameter_annotations.as_ref().unwrap()[0].len(),
        1
    );
    assert_eq!(
        method.invisible_parameter_annotations.as_ref().unwrap()[0].len(),
        1
    );
    assert!(
        matches!(image.methods()[1].attributes.annotation_default.as_ref().unwrap(), ElementValue::String(value) if value.as_utf16() == [0x61, 0, 0xd800])
    );
}

fn type_annotation_bytes(annotation: &[u8], target: &[u8]) -> Vec<u8> {
    let mut bytes = target.to_vec();
    // Every type_path alternative, including a nonzero type argument index.
    bytes.extend([4, 0, 0, 1, 0, 2, 0, 3, 2]);
    bytes.extend(annotation);
    bytes
}
fn type_annotations_bytes(annotation: &[u8], targets: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = words(&[targets.len() as u16]);
    for target in targets {
        bytes.extend(type_annotation_bytes(annotation, target));
    }
    bytes
}

#[test]
fn exposes_every_type_annotation_target_and_type_path() {
    let mut f = Fixture::new();
    let annotation = annotation_bytes(&mut f, vec![]);
    let class_targets = vec![vec![0, 0], vec![0x10, 0xff, 0xff], vec![0x11, 0, 1]];
    f.class_attr(
        "RuntimeVisibleTypeAnnotations",
        type_annotations_bytes(&annotation, &class_targets),
    );
    f.class_attr("RuntimeInvisibleTypeAnnotations", words(&[0]));
    let field = f.attr(
        "RuntimeVisibleTypeAnnotations",
        type_annotations_bytes(&annotation, &[vec![0x13]]),
    );
    let invisible = f.attr("RuntimeInvisibleTypeAnnotations", words(&[0]));
    f.field("annotated", "I", vec![field, invisible]);
    let method_targets = vec![
        vec![1, 0],
        vec![0x12, 0, 0],
        vec![0x14],
        vec![0x15],
        vec![0x16, 0],
        vec![0x17, 0, 0],
    ];
    let method = f.attr(
        "RuntimeVisibleTypeAnnotations",
        type_annotations_bytes(&annotation, &method_targets),
    );
    let invisible = f.attr("RuntimeInvisibleTypeAnnotations", words(&[0]));
    let exceptions = f.attr("Exceptions", words(&[1, f.parent]));
    f.method(
        "nativeMethod",
        "(I)V",
        0x101,
        vec![method, invisible, exceptions],
    );
    let mut targets = vec![
        vec![0x40, 0, 1, 0, 0, 0, 4, 0, 0],
        vec![0x41, 0, 1, 0, 0, 0, 4, 0, 0],
        vec![0x42, 0, 0],
    ];
    for tag in 0x43..=0x46 {
        targets.push(vec![tag, 0, 0]);
    }
    for tag in 0x47..=0x4b {
        targets.push(vec![tag, 0, 0, 2]);
    }
    let visible = f.attr(
        "RuntimeVisibleTypeAnnotations",
        type_annotations_bytes(&annotation, &targets),
    );
    let invisible = f.attr("RuntimeInvisibleTypeAnnotations", words(&[0]));
    let body = f.code(
        &[0xbb, (f.parent >> 8) as u8, f.parent as u8, 0xb1],
        1,
        1,
        &[0, 3, 3, f.parent],
        vec![visible, invisible],
    );
    f.method("codeMethod", "()V", 9, vec![body]);
    let image = f.parse();
    let class = image
        .attributes()
        .annotations
        .visible_types
        .as_ref()
        .unwrap();
    assert!(matches!(
        class[0].target,
        TypeAnnotationTarget::ClassTypeParameter(0)
    ));
    assert!(matches!(
        class[1].target,
        TypeAnnotationTarget::Supertype(65535)
    ));
    assert!(matches!(
        class[2].target,
        TypeAnnotationTarget::ClassTypeParameterBound {
            parameter: 0,
            bound: 1
        }
    ));
    assert!(
        image
            .attributes()
            .annotations
            .invisible_types
            .as_ref()
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        image.fields()[0]
            .attributes
            .annotations
            .visible_types
            .as_ref()
            .unwrap()[0]
            .target,
        TypeAnnotationTarget::Field
    ));
    assert!(
        image.fields()[0]
            .attributes
            .annotations
            .invisible_types
            .is_some()
    );
    let method = image.methods()[0]
        .attributes
        .annotations
        .visible_types
        .as_ref()
        .unwrap();
    assert!(matches!(
        method[0].target,
        TypeAnnotationTarget::MethodTypeParameter(0)
    ));
    assert!(matches!(
        method[1].target,
        TypeAnnotationTarget::MethodTypeParameterBound {
            parameter: 0,
            bound: 0
        }
    ));
    assert!(matches!(
        method[2].target,
        TypeAnnotationTarget::MethodReturn
    ));
    assert!(matches!(
        method[3].target,
        TypeAnnotationTarget::MethodReceiver
    ));
    assert!(matches!(
        method[4].target,
        TypeAnnotationTarget::MethodParameter(0)
    ));
    assert!(matches!(method[5].target, TypeAnnotationTarget::Throws(0)));
    let code = code(&image, 1)
        .attributes
        .visible_type_annotations
        .as_ref()
        .unwrap();
    assert!(
        matches!(&code[0].target, TypeAnnotationTarget::LocalVariable(ranges) if ranges[0].length == 4)
    );
    assert!(
        matches!(&code[1].target, TypeAnnotationTarget::ResourceVariable(ranges) if ranges[0].index == 0)
    );
    assert!(matches!(code[2].target, TypeAnnotationTarget::Catch(0)));
    for (i, tag) in (0x43..=0x46).enumerate() {
        assert!(
            matches!(code[i+3].target, TypeAnnotationTarget::Offset { target_type, offset: 0 } if target_type == tag)
        );
    }
    for (i, tag) in (0x47..=0x4b).enumerate() {
        assert!(
            matches!(code[i+7].target, TypeAnnotationTarget::TypeArgument { target_type, offset: 0, argument: 2 } if target_type == tag)
        );
    }
    assert!(matches!(
        code[0].path.as_slice(),
        [
            TypePathStep::ArrayElement,
            TypePathStep::NestedType,
            TypePathStep::WildcardBound,
            TypePathStep::TypeArgument(2)
        ]
    ));
}

#[test]
fn exposes_stack_map_forms_and_repeated_debug_tables_with_original_offsets() {
    let mut f = Fixture::new();
    let mut frames = words(&[10]);
    frames.push(0); // same, offset 0
    frames.extend([64, 1]); // one integer, offset 1
    frames.push(247);
    frames.extend(words(&[0]));
    frames.push(2); // one float, offset 2
    frames.push(252);
    frames.extend(words(&[0]));
    frames.push(4); // append long, offset 3
    frames.push(250);
    frames.extend(words(&[0])); // chop one, offset 4
    frames.push(251);
    frames.extend(words(&[0])); // extended same, offset 5
    frames.push(255);
    frames.extend(words(&[0, 3]));
    frames.push(7);
    frames.extend(words(&[f.parent]));
    frames.push(8);
    frames.extend(words(&[20]));
    frames.push(6);
    frames.extend(words(&[2]));
    frames.extend([3, 5]); // full: object, uninit, uninitThis; double, null
    frames.push(253);
    frames.extend(words(&[0]));
    frames.extend([0, 1]); // append top, int, offset 7
    frames.push(249);
    frames.extend(words(&[0])); // chop two, offset 8
    frames.push(248);
    frames.extend(words(&[0])); // chop three, offset 9
    let stack = f.attr("StackMapTable", frames);
    let first_lines = f.attr("LineNumberTable", words(&[2, 0, 12, 20, 13]));
    let second_lines = f.attr("LineNumberTable", words(&[1, 23, 14]));
    let name = f.utf8("value");
    let descriptor = f.utf8("J");
    let object = f.utf8("Ljava/lang/Object;");
    let signature = f.utf8("TT;");
    let first_locals = f.attr(
        "LocalVariableTable",
        words(&[1, 0, 24, name, descriptor, 0]),
    );
    let second_locals = f.attr("LocalVariableTable", words(&[1, 0, 24, name, object, 2]));
    let first_types = f.attr(
        "LocalVariableTypeTable",
        words(&[1, 0, 24, name, signature, 2]),
    );
    let second_types = f.attr("LocalVariableTypeTable", words(&[0]));
    let unknown = f.attr("VendorCode", words(&[name]));
    let mut bytes = vec![0; 20];
    bytes.push(0xbb);
    bytes.extend(words(&[f.parent]));
    bytes.push(0xb1);
    let body = f.code(
        &bytes,
        8,
        8,
        &[],
        vec![
            stack,
            first_lines,
            second_lines,
            first_locals,
            second_locals,
            first_types,
            second_types,
            unknown,
        ],
    );
    f.method("run", "()V", 9, vec![body]);
    let image = f.parse();
    let attrs = &code(&image, 0).attributes;
    let frames = attrs.stack_map_table.as_ref().unwrap();
    assert_eq!(
        frames.iter().map(|frame| frame.offset).collect::<Vec<_>>(),
        (0..10).collect::<Vec<_>>()
    );
    assert!(matches!(frames[0].kind, StackMapFrameKind::Same));
    assert!(matches!(
        frames[1].kind,
        StackMapFrameKind::SameLocalsOneStack(VerificationType::Integer)
    ));
    assert!(matches!(
        frames[2].kind,
        StackMapFrameKind::SameLocalsOneStack(VerificationType::Float)
    ));
    assert!(matches!(frames[3].kind, StackMapFrameKind::Append(_)));
    assert!(matches!(frames[4].kind, StackMapFrameKind::Chop(1)));
    assert!(matches!(frames[5].kind, StackMapFrameKind::Same));
    let StackMapFrameKind::Full { locals, stack } = &frames[6].kind else {
        panic!()
    };
    assert!(matches!(locals[1], VerificationType::Uninitialized(20)));
    assert!(matches!(locals[2], VerificationType::UninitializedThis));
    assert!(matches!(
        stack.as_slice(),
        [VerificationType::Double, VerificationType::Null]
    ));
    assert!(matches!(frames[8].kind, StackMapFrameKind::Chop(2)));
    assert!(matches!(frames[9].kind, StackMapFrameKind::Chop(3)));
    assert_eq!(attrs.line_number_tables.len(), 2);
    assert_eq!(attrs.line_number_tables[0][1].start_pc, 20);
    assert_eq!(attrs.line_number_tables[1][0].line_number, 14);
    assert_eq!(attrs.local_variable_tables.len(), 2);
    assert_eq!(attrs.local_variable_tables[0][0].descriptor.raw(), "J");
    assert!(attrs.local_variable_tables[0][0].name.eq_str("value"));
    assert_eq!(attrs.local_variable_tables[1][0].range.index, 2);
    assert_eq!(attrs.local_variable_type_tables.len(), 2);
    assert!(
        attrs.local_variable_type_tables[0][0]
            .signature
            .eq_str("TT;")
    );
    assert!(attrs.local_variable_type_tables[0][0].name.eq_str("value"));
    assert_eq!(attrs.local_variable_type_tables[0][0].range.length, 24);
    assert_eq!(&*attrs.unknown[0].body, words(&[name]));
}

fn assert_invalid(f: &Fixture) {
    assert!(
        parse_class(&f.bytes()).is_err(),
        "malformed class was accepted"
    );
}

#[test]
fn rejects_bad_attribute_names_lengths_duplicates_and_truncation() {
    for index in [0, u16::MAX, 2] {
        let mut f = Fixture::new();
        f.attrs.push(Attr {
            name: index,
            body: vec![],
        });
        assert_invalid(&f);
    }
    for body in [vec![], vec![0], vec![0, 0], vec![0, 1, 0]] {
        let mut f = Fixture::new();
        f.class_attr("SourceFile", body);
        assert_invalid(&f);
    }
    let mut f = Fixture::new();
    let text = f.utf8("Test.java");
    f.class_attr("SourceFile", words(&[text]));
    f.class_attr("SourceFile", words(&[text]));
    assert_invalid(&f);
    for name in ["Synthetic", "Deprecated"] {
        let mut f = Fixture::new();
        f.class_attr(name, vec![0]);
        assert_invalid(&f);
        let mut f = Fixture::new();
        f.class_attr(name, vec![]);
        f.class_attr(name, vec![]);
        assert_invalid(&f);
    }
    let mut f = Fixture::new();
    let n = f.integer(1);
    let constant = f.attr("ConstantValue", words(&[n]));
    f.field("n", "I", vec![constant.clone(), constant]);
    assert_invalid(&f);
    let mut f = Fixture::new();
    let c = f.code(&[0xb1], 0, 0, &[], vec![]);
    f.method("run", "()V", 9, vec![c.clone(), c]);
    assert_invalid(&f);
    let mut f = Fixture::new();
    let table = f.attr("StackMapTable", words(&[0]));
    let c = f.code(&[0xb1], 0, 0, &[], vec![table.clone(), table]);
    f.method("run", "()V", 9, vec![c]);
    assert_invalid(&f);
    // A populated class, including nested attributes, must reject every prefix.
    let mut f = Fixture::new();
    let lines = f.attr("LineNumberTable", words(&[1, 0, 12]));
    let code = f.code(&[0xb1], 0, 0, &[], vec![lines]);
    f.method("run", "()V", 9, vec![code]);
    f.class_attr("RuntimeVisibleAnnotations", words(&[0]));
    let bytes = f.bytes();
    for length in 0..bytes.len() {
        assert!(
            parse_class(&bytes[..length]).is_err(),
            "accepted prefix {length}"
        );
    }
}

#[test]
fn rejects_wrong_constant_kinds_and_invalid_bootstrap_links_including_unused_entries() {
    for descriptor in ["J", "D", "F", "[I", "Ljava/lang/Object;"] {
        let mut f = Fixture::new();
        let n = f.integer(1);
        let constant = f.attr("ConstantValue", words(&[n]));
        f.field("wrong", descriptor, vec![constant]);
        assert_invalid(&f);
    }
    let mut f = Fixture::new();
    let text = f.utf8("not a CONSTANT_String");
    let constant = f.attr("ConstantValue", words(&[text]));
    f.field("wrong", "Ljava/lang/String;", vec![constant]);
    assert_invalid(&f);
    for table in [None, Some(vec![0, 0])] {
        let mut f = Fixture::new();
        let nat = f.nat("call", "()V");
        let mut entry = vec![18];
        entry.extend(words(&[0, nat]));
        f.entry(entry);
        if let Some(table) = table {
            f.class_attr("BootstrapMethods", table);
        }
        assert_invalid(&f);
    }
    let mut f = Fixture::new();
    let integer = f.integer(1);
    f.class_attr("BootstrapMethods", words(&[1, integer, 0]));
    assert_invalid(&f);
    let mut f = Fixture::new();
    let member = f.method_ref();
    let handle = f.handle(member);
    let text = f.utf8("not loadable");
    f.class_attr("BootstrapMethods", words(&[1, handle, 1, text]));
    assert_invalid(&f);
    let mut f = Fixture::new();
    let member = f.method_ref();
    let mut handle = vec![15, 5];
    u2(&mut handle, member);
    let handle = f.entry(handle);
    f.class_attr("BootstrapMethods", words(&[1, handle, 0]));
    assert_invalid(&f);
}

#[test]
fn rejects_malformed_annotations_type_targets_paths_and_excessive_nesting() {
    for tag in [b'?', b'I', b's', b'e', b'c', b'@', b'['] {
        let mut f = Fixture::new();
        let bytes = annotation_bytes(&mut f, vec![("value", vec![tag])]);
        f.class_attr("RuntimeVisibleAnnotations", counted(&bytes));
        assert_invalid(&f);
    }
    let mut f = Fixture::new();
    let float = f.numeric(4, 0);
    let bytes = annotation_bytes(&mut f, vec![("value", tagged(b'I', float))]);
    f.class_attr("RuntimeVisibleAnnotations", counted(&bytes));
    assert_invalid(&f);
    let mut f = Fixture::new();
    let number = f.integer(1);
    let bytes = annotation_bytes(
        &mut f,
        vec![
            ("value", tagged(b'I', number)),
            ("value", tagged(b'I', number)),
        ],
    );
    f.class_attr("RuntimeVisibleAnnotations", counted(&bytes));
    assert_invalid(&f);
    let mut f = Fixture::new();
    let mut value = Vec::new();
    for _ in 0..140 {
        value.extend([b'[', 0, 1]);
    }
    value.extend([b'[', 0, 0]);
    let bytes = annotation_bytes(&mut f, vec![("value", value)]);
    f.class_attr("RuntimeVisibleAnnotations", counted(&bytes));
    assert_invalid(&f);
    for target in [vec![0x13], vec![0x7f], vec![0x00]] {
        let mut f = Fixture::new();
        let annotation = annotation_bytes(&mut f, vec![]);
        f.class_attr(
            "RuntimeVisibleTypeAnnotations",
            type_annotations_bytes(&annotation, &[target]),
        );
        assert_invalid(&f);
    }
    for path in [[0, 1], [1, 1], [2, 1], [4, 0]] {
        let mut f = Fixture::new();
        let annotation = annotation_bytes(&mut f, vec![]);
        let mut body = words(&[1]);
        body.extend([0, 0, 1]);
        body.extend(path);
        body.extend(annotation);
        f.class_attr("RuntimeVisibleTypeAnnotations", body);
        assert_invalid(&f);
    }
}

fn class_with_code_attribute(
    name: &str,
    body: Vec<u8>,
    max_stack: u16,
    max_locals: u16,
) -> Fixture {
    let mut f = Fixture::new();
    let attr = f.attr(name, body);
    // bipush occupies two bytes; offset 1 is not an instruction boundary.
    let code = f.code(&[0x10, 0, 0xb1], max_stack, max_locals, &[], vec![attr]);
    f.method("run", "()V", 9, vec![code]);
    f
}

#[test]
fn rejects_invalid_stack_map_tags_offsets_types_and_capacities() {
    for body in [
        vec![0, 1, 128],                      // reserved frame tag
        vec![0, 1, 64, 9],                    // unknown verification type
        vec![0, 1, 1],                        // offset in middle of bipush
        vec![0, 1, 3],                        // offset == code_length
        vec![0, 2, 0, 251, 255, 255],         // offset overflow
        vec![0, 1, 64, 7, 0, 0],              // invalid Object CP index
        vec![0, 1, 64, 8, 0, 0],              // uninitialized offset identifies bipush, not new
        vec![0, 1, 64, 4],                    // long exceeds max_stack=1
        vec![0, 1, 255, 0, 0, 0, 1, 4, 0, 0], // long local exceeds max_locals=1
        vec![0, 1, 255, 0, 0, 0, 0, 0, 1, 4], // long stack exceeds max_stack=1
    ] {
        assert_invalid(&class_with_code_attribute("StackMapTable", body, 1, 1));
    }
}

#[test]
fn rejects_invalid_debug_ranges_and_code_type_annotation_offsets() {
    for offset in [1, 3, u16::MAX] {
        assert_invalid(&class_with_code_attribute(
            "LineNumberTable",
            words(&[1, offset, 10]),
            1,
            1,
        ));
    }
    for (start, length, index, descriptor) in [
        (1, 1, 0, "I"),
        (0, 1, 0, "I"),
        (0, 4, 0, "I"),
        (0, 3, 1, "I"),
        (0, 3, 0, "J"),
    ] {
        let mut f = Fixture::new();
        let name = f.utf8("n");
        let desc = f.utf8(descriptor);
        let attr = f.attr(
            "LocalVariableTable",
            words(&[1, start, length, name, desc, index]),
        );
        let body = f.code(&[0x10, 0, 0xb1], 1, 1, &[], vec![attr]);
        f.method("run", "()V", 9, vec![body]);
        assert_invalid(&f);
    }
    for target in [
        vec![0x43, 0, 1],
        vec![0x47, 0, 3, 0],
        vec![0x42, 0, 0],
        vec![0x40, 0, 1, 0, 0, 0, 4, 0, 0],
    ] {
        let mut f = Fixture::new();
        let annotation = annotation_bytes(&mut f, vec![]);
        let attr = f.attr(
            "RuntimeVisibleTypeAnnotations",
            type_annotations_bytes(&annotation, &[target]),
        );
        let body = f.code(&[0x10, 0, 0xb1], 1, 1, &[], vec![attr]);
        f.method("run", "()V", 9, vec![body]);
        assert_invalid(&f);
    }
}

#[test]
fn validates_code_presence_and_preserves_zero_optional_indices() {
    for flags in [0x109, 0x401] {
        let mut f = Fixture::new();
        let body = f.code(&[0xb1], 0, 0, &[], vec![]);
        f.method("run", "()V", flags, vec![body]);
        assert_invalid(&f);
    }
    let mut f = Fixture::new();
    f.method("run", "()V", 9, vec![]);
    assert_invalid(&f);
    let mut f = Fixture::new();
    f.class_attr("EnclosingMethod", words(&[f.parent, 0]));
    let parameter = f.attr("MethodParameters", vec![1, 0, 0, 0x90, 0x10]);
    f.method("run", "(I)V", 0x109, vec![parameter]);
    let image = f.parse();
    assert!(
        image
            .attributes()
            .enclosing_method
            .as_ref()
            .unwrap()
            .method
            .is_none()
    );
    assert!(
        image.methods()[0].attributes.parameters.as_ref().unwrap()[0]
            .name
            .is_none()
    );
    for body in [vec![0], vec![1, 0, 0, 0, 1]] {
        let mut f = Fixture::new();
        let parameter = f.attr("MethodParameters", body);
        f.method("run", "(I)V", 0x109, vec![parameter]);
        assert_invalid(&f);
    }
}

#[test]
fn validates_stack_map_local_changes_using_method_parameter_widths() {
    // Chopping one verification entry removes a long's two local slots.
    let mut f = Fixture::new();
    let table = f.attr("StackMapTable", vec![0, 2, 250, 0, 0, 252, 0, 0, 4]);
    let body = f.code(&[0, 0xb1], 0, 2, &[], vec![table]);
    f.method("wide", "(J)V", 9, vec![body]);
    f.parse();
    for (descriptor, flags, max_locals, frames) in [
        ("()V", 9, 1, vec![0, 1, 250, 0, 0]), // chop with no initial locals
        ("()V", 9, 1, vec![0, 1, 252, 0, 0, 4]), // append long into one slot
        ("(J)V", 9, 2, vec![0, 1, 252, 0, 0, 1]), // parameters already occupy capacity
        ("()V", 1, 1, vec![0, 1, 252, 0, 0, 1]), // receiver already occupies capacity
    ] {
        let mut f = Fixture::new();
        let table = f.attr("StackMapTable", frames);
        let body = f.code(&[0xb1], 0, max_locals, &[], vec![table]);
        f.method("run", descriptor, flags, vec![body]);
        assert_invalid(&f);
    }
    for (descriptor, flags, max_locals) in [("(J)V", 9, 1), ("(I)V", 1, 1)] {
        let mut f = Fixture::new();
        let body = f.code(&[0xb1], 0, max_locals, &[], vec![]);
        f.method("run", descriptor, flags, vec![body]);
        assert_invalid(&f);
    }
}

#[test]
fn validates_declaration_type_annotation_indices_and_receiver_location() {
    for target in [vec![0x10, 0, 0], vec![0x10, 0xff, 0xff]] {
        let mut f = Fixture::new();
        let annotation = annotation_bytes(&mut f, vec![]);
        if target[1] == 0xff {
            f.parent = 0;
        }
        f.class_attr(
            "RuntimeVisibleTypeAnnotations",
            type_annotations_bytes(&annotation, &[target]),
        );
        assert_invalid(&f);
    }
    for target in [vec![0x16, 1], vec![0x17, 0, 0], vec![0x15]] {
        let mut f = Fixture::new();
        let annotation = annotation_bytes(&mut f, vec![]);
        let attr = f.attr(
            "RuntimeVisibleTypeAnnotations",
            type_annotations_bytes(&annotation, &[target]),
        );
        f.method("run", "(I)V", 0x109, vec![attr]);
        assert_invalid(&f);
    }
}

#[test]
fn preserves_attributes_outside_their_predefined_location_as_opaque() {
    let mut f = Fixture::new();
    // Names are reserved in their specified context. Do not interpret a vendor
    // body as Code on a field or ConstantValue on a class.
    f.class_attr("ConstantValue", vec![1]);
    let attr = f.attr("Code", vec![2]);
    f.field("n", "I", vec![attr]);
    let image = f.parse();
    assert_eq!(&*image.attributes().unknown[0].body, [1]);
    assert_eq!(&*image.fields()[0].attributes.unknown[0].body, [2]);
}
