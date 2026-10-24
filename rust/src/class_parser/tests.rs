use super::*;

/// Minimal class-file writer used by parser tests.
pub(super) struct ClassWriter {
    bytes: Vec<u8>,
}

impl ClassWriter {
    pub(super) fn new() -> Self {
        Self {
            bytes: vec![0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 52],
        }
    }

    pub(super) fn u1(&mut self, value: u8) -> &mut Self {
        self.bytes.push(value);
        self
    }

    pub(super) fn u2(&mut self, value: u16) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub(super) fn u4(&mut self, value: u32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub(super) fn bytes(&mut self, value: &[u8]) -> &mut Self {
        self.bytes.extend_from_slice(value);
        self
    }

    pub(super) fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

/// One method, no fields, no interfaces, with optional Code bytes.
pub(super) fn simple_class(
    name: &str,
    method_name: &str,
    descriptor: &str,
    method_flags: u16,
    code: &[u8],
    max_stack: u16,
    max_locals: u16,
) -> Vec<u8> {
    let mut w = ClassWriter::new();
    w.u2(8);
    // #1 Utf8 name
    w.u1(1).u2(name.len() as u16).bytes(name.as_bytes());
    // #2 Class #1
    w.u1(7).u2(1);
    // #3 Utf8 java/lang/Object
    let object = "java/lang/Object";
    w.u1(1).u2(object.len() as u16).bytes(object.as_bytes());
    // #4 Class #3
    w.u1(7).u2(3);
    // #5 Utf8 method name
    w.u1(1)
        .u2(method_name.len() as u16)
        .bytes(method_name.as_bytes());
    // #6 Utf8 descriptor
    w.u1(1)
        .u2(descriptor.len() as u16)
        .bytes(descriptor.as_bytes());
    // #7 Utf8 Code
    w.u1(1).u2(4).bytes(b"Code");

    // access, this, super, interfaces
    w.u2(0x0021).u2(2).u2(4).u2(0);
    // fields
    w.u2(0);
    // methods
    w.u2(1);
    w.u2(method_flags).u2(5).u2(6).u2(1);
    w.u2(7); // Code
    let code_len = code.len() as u32;
    w.u4(2 + 2 + 4 + code_len + 2 + 2);
    w.u2(max_stack).u2(max_locals).u4(code_len).bytes(code);
    w.u2(0); // exception table
    w.u2(0); // nested attributes
    // class attributes
    w.u2(0);
    w.finish()
}

#[test]
fn parses_minimal_class_and_code() {
    let bytes = simple_class("Test", "run", "()V", 0x0009, &[0xb1], 0, 0);
    let image = parse_class(&bytes).unwrap();
    assert_eq!(image.this_class_name(), "Test");
    assert_eq!(image.super_class_name(), Some("java/lang/Object"));
    assert_eq!(
        image.access_flags(),
        ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER
    );
    assert_eq!(image.methods().len(), 1);
    let method = &image.methods()[0];
    assert_eq!(method.name, "run");
    assert_eq!(method.descriptor.raw(), "()V");
    assert!(method.access_flags.contains(MethodAccessFlags::STATIC));
    let code = method.code.as_ref().unwrap();
    assert_eq!(code.bytecode.opcodes.len(), 1);
    assert!(matches!(code.bytecode.opcodes[0].1, Opcode::Return));
}

#[test]
fn rejects_truncation_without_panicking() {
    let bytes = simple_class("Test", "run", "()V", 0x0009, &[0xb1], 0, 0);
    for end in 0..bytes.len() {
        assert!(
            parse_class(&bytes[..end]).is_err(),
            "prefix of length {end} was accepted"
        );
    }
}

#[test]
fn preserves_methodref_and_interfacemethodref_tags() {
    let mut w = ClassWriter::new();
    w.u2(13);
    // #1 Utf8 java/lang/Object
    let object = "java/lang/Object";
    w.u1(1).u2(object.len() as u16).bytes(object.as_bytes());
    // #2 Class #1
    w.u1(7).u2(1);
    // #3 Utf8 m
    w.u1(1).u2(1).bytes(b"m");
    // #4 Utf8 ()V
    w.u1(1).u2(3).bytes(b"()V");
    // #5 NameAndType #3:#4
    w.u1(12).u2(3).u2(4);
    // #6 Methodref #2:#5
    w.u1(10).u2(2).u2(5);
    // #7 InterfaceMethodref #2:#5
    w.u1(11).u2(2).u2(5);
    // #8 Utf8 Test
    w.u1(1).u2(4).bytes(b"Test");
    // #9 Class #8
    w.u1(7).u2(8);
    // #10 Utf8 run
    w.u1(1).u2(3).bytes(b"run");
    // #11 Utf8 ()V
    w.u1(1).u2(3).bytes(b"()V");
    // #12 Utf8 Code
    w.u1(1).u2(4).bytes(b"Code");

    w.u2(0x0021).u2(9).u2(2).u2(0);
    w.u2(0); // fields
    w.u2(1); // methods
    w.u2(0x0009).u2(10).u2(11).u2(1);
    w.u2(12);
    let code = [
        0xb8, 0x00, 0x06, // invokestatic #6 (Methodref)
        0xb8, 0x00, 0x07, // invokestatic #7 (InterfaceMethodref)
        0xb1,
    ];
    w.u4(2 + 2 + 4 + code.len() as u32 + 2 + 2);
    w.u2(0).u2(0).u4(code.len() as u32).bytes(&code);
    w.u2(0).u2(0);
    w.u2(0);

    let image = parse_class(&w.finish()).unwrap();
    let code = image.methods()[0].code.as_ref().unwrap();
    let Opcode::InvokeStatic(class_ref) = &code.bytecode.opcodes[0].1 else {
        panic!("expected invokestatic");
    };
    assert_eq!(class_ref.kind, MemberKind::Method);
    assert_eq!(class_ref.owner, "java/lang/Object");
    assert_eq!(class_ref.name, "m");
    assert_eq!(class_ref.descriptor, "()V");

    let Opcode::InvokeStatic(interface_ref) = &code.bytecode.opcodes[1].1 else {
        panic!("expected invokestatic");
    };
    assert_eq!(interface_ref.kind, MemberKind::InterfaceMethod);
    assert_eq!(interface_ref.owner, "java/lang/Object");
    assert_eq!(interface_ref.name, "m");
    assert_eq!(interface_ref.descriptor, "()V");

    let entries: Vec<_> = image.constant_pool().collect();
    assert_eq!(entries.len(), 12);
    assert_eq!(entries[0].0, 1);
    assert!(matches!(entries[0].1, Constant::Utf8(_)));
    assert!(
        entries
            .iter()
            .any(|(_, entry)| matches!(entry, Constant::MethodRef { .. }))
    );
    assert!(
        entries
            .iter()
            .any(|(_, entry)| matches!(entry, Constant::InterfaceMethodRef { .. }))
    );
}

#[test]
fn rejects_unrepresentable_names_with_bytes_in_the_error() {
    // Replace the method name with an unpaired high surrogate.
    let mut bytes = simple_class("Test", "run", "()V", 0x0009, &[0xb1], 0, 0);
    // #5 starts after the fixed 8-byte header, #1, #2, #3, #4.
    // Find the method name Utf8 entry by locating the 0x01 tag followed by
    // length 3 and "run". It is unique in this fixture.
    let needle = [0x01, 0x00, 0x03, b'r', b'u', b'n'];
    let at = bytes
        .windows(needle.len())
        .position(|window| window == needle)
        .expect("method name entry");
    bytes[at + 3..at + 6].copy_from_slice(&[0xED, 0xA0, 0x80]);
    let error = parse_class(&bytes).unwrap_err();
    match error.kind {
        ParseErrorKind::InvalidText { bytes } => {
            assert_eq!(&*bytes, &[0xED, 0xA0, 0x80]);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn rejects_unsupported_versions() {
    let mut bytes = simple_class("Test", "run", "()V", 0x0009, &[0xb1], 0, 0);
    bytes[6] = 0;
    bytes[7] = 51;
    assert!(matches!(
        parse_class(&bytes).unwrap_err().kind,
        ParseErrorKind::UnsupportedVersion { major: 51, .. }
    ));
}

#[test]
fn parses_fields() {
    let mut w = ClassWriter::new();
    w.u2(7);
    // #1 Utf8 Test, #2 Class, #3 Utf8 Object, #4 Class,
    // #5 Utf8 value, #6 Utf8 I
    w.u1(1).u2(4).bytes(b"Test");
    w.u1(7).u2(1);
    let object = "java/lang/Object";
    w.u1(1).u2(object.len() as u16).bytes(object.as_bytes());
    w.u1(7).u2(3);
    w.u1(1).u2(5).bytes(b"value");
    w.u1(1).u2(1).bytes(b"I");

    w.u2(0x0021).u2(2).u2(4).u2(0);
    w.u2(1); // one field
    w.u2(0x0002).u2(5).u2(6).u2(0);
    w.u2(0); // no methods
    w.u2(0); // no class attributes

    let image = parse_class(&w.finish()).unwrap();
    assert_eq!(image.fields().len(), 1);
    let field = &image.fields()[0];
    assert_eq!(field.name, "value");
    assert_eq!(field.descriptor.raw(), "I");
    assert_eq!(field.access_flags.bits(), 0x0002);
}
