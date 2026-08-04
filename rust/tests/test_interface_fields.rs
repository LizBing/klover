mod test_harness;

use klover::{class_loader::ms_api::MSRef, oops::normal_klass::NormalKlass};
use test_harness::load_class;

fn getstatic_index(holder: &MSRef<NormalKlass>, method_name: &str) -> usize {
    let method = holder
        .find_declared_method(method_name, "()I")
        .unwrap_or_else(|| panic!("method not found: {method_name}()I"));
    let bytecodes = &method.code.as_ref().expect("method has no Code").bytecodes;
    let opcode = bytecodes
        .iter()
        .position(|byte| *byte == 0xb2)
        .expect("method contains no getstatic");

    u16::from_be_bytes([bytecodes[opcode + 1], bytecodes[opcode + 2]]) as usize
}

fn assert_resolves_to_root(holder: &MSRef<NormalKlass>, method_name: &str) {
    let index = getstatic_index(holder, method_name);
    let resolved = holder.resolve_field_ref(index).unwrap();
    let root = load_class("RootInterfaceField");

    assert!(resolved.holder.equals(&root));
    assert_eq!(resolved.field.name.utf8(), "VALUE");
}

#[test]
fn field_resolution_searches_parent_interfaces_recursively() {
    let holder = load_class("InterfaceStaticFields");
    assert_resolves_to_root(&holder, "readThroughMiddle");
}

#[test]
fn field_resolution_handles_a_shared_parent_interface() {
    let holder = load_class("InterfaceStaticFields");
    assert_resolves_to_root(&holder, "readThroughClass");
}
