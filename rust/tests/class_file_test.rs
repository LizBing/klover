use std::{fs::File, io::Read};

use rust::class_parser::{
    acc_flags::AccFlags, attr_info::AttrInfo, class_file::ClassFileBuilder,
    cp_info::ConstantPoolInfo,
};

fn read_class_bytes(path: &str) -> Vec<u8> {
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    buf
}

#[test]
fn test_hello_klover_builder() {
    let buf = read_class_bytes("../test_data/classes/HelloKlover.class");

    let builder = ClassFileBuilder::from(&buf).unwrap();

    // --- version ---
    assert_eq!(builder.minor_version, 0);
    assert_eq!(builder.major_version, 69);

    // --- access flags ---
    assert!(builder.acc_flags.contains(AccFlags::ACC_PUBLIC));
    assert!(builder.acc_flags.contains(AccFlags::ACC_SUPER));

    // --- constant pool count (javap says 28 entries, #1 to #28, cp_count=29) ---
    assert_eq!(builder.constant_pool.len(), 28);

    // --- resolve this_class and super_class via cp ---
    let this_entry = &builder.constant_pool[builder.this_class as usize - 1];
    let super_entry = &builder.constant_pool[builder.super_class as usize - 1];

    assert!(matches!(this_entry, ConstantPoolInfo::ClassInfo { .. }));
    assert!(matches!(super_entry, ConstantPoolInfo::ClassInfo { .. }));

    // --- interfaces (none) ---
    assert!(builder.interfaces.is_empty());

    // --- fields (none) ---
    assert!(builder.fields.is_empty());

    // --- methods (2: <init> and main) ---
    assert_eq!(builder.methods.len(), 2);

    // --- attributes (1: SourceFile) ---
    assert_eq!(builder.attributes.len(), 1);
    assert!(matches!(builder.attributes[0], AttrInfo::Unparsed { .. }));

    // --- build into ClassFile ---
    let class_file = builder.build();

    assert_eq!(class_file.this_class, "HelloKlover");
    assert_eq!(class_file.super_class.as_deref(), Some("java/lang/Object"));
    assert!(class_file.super_class.is_some());
    assert_eq!(class_file.methods.len(), 2);
    assert_eq!(class_file.fields.len(), 0);
    assert_eq!(class_file.interfaces.len(), 0);
    assert_eq!(class_file.attributes.len(), 1);
}

#[test]
fn test_hello_klover_cp_entries() {
    let buf = read_class_bytes("../test_data/classes/HelloKlover.class");
    let builder = ClassFileBuilder::from(&buf).unwrap();

    // cp[0] = #1 Methodref java/lang/Object."<init>":()V
    assert!(matches!(
        builder.constant_pool[0],
        ConstantPoolInfo::MethodrefInfo {
            class_index: 2,
            name_and_type_index: 3
        }
    ));

    // cp[1] = #2 Class java/lang/Object
    assert!(matches!(
        builder.constant_pool[1],
        ConstantPoolInfo::ClassInfo { name_index: 4 }
    ));

    // cp[2] = #3 NameAndType "<init>":()V
    assert!(matches!(
        builder.constant_pool[2],
        ConstantPoolInfo::NameAndTypeInfo {
            name_index: 5,
            desc_index: 6
        }
    ));

    // cp[3] = #4 Utf8 "java/lang/Object"
    assert!(matches!(
        builder.constant_pool[3],
        ConstantPoolInfo::Utf8Info { .. }
    ));

    // cp[5] = #6 Utf8 "()V"
    assert!(matches!(
        builder.constant_pool[5],
        ConstantPoolInfo::Utf8Info { .. }
    ));

    // cp[12] = #13 String "Hello Klover!"
    assert!(matches!(
        builder.constant_pool[12],
        ConstantPoolInfo::StringInfo { string_index: 14 }
    ));

    // cp[27] = #28 Utf8 "HelloKlover.java"
    assert!(matches!(
        builder.constant_pool[27],
        ConstantPoolInfo::Utf8Info { .. }
    ));

    // Verify some specific Utf8 content
    if let ConstantPoolInfo::Utf8Info { utf8 } = &builder.constant_pool[3] {
        assert_eq!(utf8, "java/lang/Object");
    } else {
        panic!("cp[4] should be Utf8");
    }
}

#[test]
fn test_hello_klover_methods() {
    let buf = read_class_bytes("../test_data/classes/HelloKlover.class");
    let builder = ClassFileBuilder::from(&buf).unwrap();

    // method[0] = <init>
    let init = &builder.methods[0];
    assert!(init.acc_flags.contains(AccFlags::ACC_PUBLIC));

    // method[1] = main
    let main = &builder.methods[1];
    assert!(main.acc_flags.contains(AccFlags::ACC_PUBLIC));
    assert!(main.acc_flags.contains(AccFlags::ACC_STATIC));
}

#[test]
fn test_hello_klover_sourcefile_attr() {
    let buf = read_class_bytes("../test_data/classes/HelloKlover.class");
    let builder = ClassFileBuilder::from(&buf).unwrap();

    // Last attribute should be SourceFile pointing to "HelloKlover.java"
    assert_eq!(builder.attributes.len(), 1);

    match &builder.attributes[0] {
        AttrInfo::Unparsed { name_index, data } => {
            // name_index points to cp entry #27 = "SourceFile"
            let name_entry = &builder.constant_pool[*name_index as usize - 1];
            if let ConstantPoolInfo::Utf8Info { utf8 } = name_entry {
                assert_eq!(utf8, "SourceFile");
            } else {
                panic!("attribute name not Utf8");
            }

            // data: u2 sourcefile_index, which points to cp entry #28 = "HelloKlover.java"
            assert_eq!(data.len(), 2);
            let sf_idx = u16::from_be_bytes([data[0], data[1]]);
            let sf_entry = &builder.constant_pool[sf_idx as usize - 1];
            if let ConstantPoolInfo::Utf8Info { utf8 } = sf_entry {
                assert_eq!(utf8, "HelloKlover.java");
            } else {
                panic!("SourceFile index not Utf8");
            }
        }
        _ => panic!("expected Unparsed attribute"),
    }
}

#[test]
fn test_invalid_magic() {
    let buf = vec![0x00, 0x00, 0x00, 0x00];
    let result = ClassFileBuilder::from(&buf);
    assert!(result.is_err());
}
