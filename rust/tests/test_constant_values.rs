mod test_harness;

use klover::{
    class_loader::ms_api::MSRef,
    engine::{exec_error::ExecError, slot::Slot},
    oops::{normal_klass::NormalKlass, symbol_table::SymbolTable},
};
use test_harness::{expect_int, load_class, run, try_run};

fn read_constant(holder: &MSRef<NormalKlass>, name: &str, desc: &str) -> Vec<Slot> {
    let name = SymbolTable::intern(name);
    let desc = SymbolTable::intern(desc);
    let field = holder
        .find_declared_field_symbol(&name, &desc)
        .unwrap_or_else(|| panic!("constant field not found: {}", name.utf8()));

    holder.read_static_field(&field).unwrap()
}

#[test]
fn numeric_constant_values_are_installed_during_class_initialization() {
    let holder = load_class("NumericConstantValues");
    assert_eq!(
        expect_int(run(&holder, "triggerInitialization", "()I", vec![],)),
        0
    );

    assert_eq!(
        read_constant(&holder, "BOOLEAN_VALUE", "Z")[0]
            .as_int()
            .unwrap(),
        1
    );
    assert_eq!(
        read_constant(&holder, "BYTE_VALUE", "B")[0]
            .as_int()
            .unwrap(),
        -7
    );
    assert_eq!(
        read_constant(&holder, "CHAR_VALUE", "C")[0]
            .as_int()
            .unwrap(),
        65_530
    );
    assert_eq!(
        read_constant(&holder, "SHORT_VALUE", "S")[0]
            .as_int()
            .unwrap(),
        -1_234
    );
    assert_eq!(
        read_constant(&holder, "INT_VALUE", "I")[0]
            .as_int()
            .unwrap(),
        123_456_789
    );
    assert_eq!(
        read_constant(&holder, "FLOAT_VALUE", "F")[0]
            .as_float()
            .unwrap(),
        3.25
    );

    let long = read_constant(&holder, "LONG_VALUE", "J");
    assert_eq!(Slot::as_long(long[0], long[1]).unwrap(), 0x1_2345_6789);

    let double = read_constant(&holder, "DOUBLE_VALUE", "D");
    assert_eq!(Slot::as_double(double[0], double[1]).unwrap(), -1234.5);
}

#[test]
fn string_constant_value_fails_explicitly_until_java_strings_exist() {
    let holder = load_class("StringConstantValue");

    for _ in 0..2 {
        let error = try_run(&holder, "triggerInitialization", "()I", vec![]).unwrap_err();
        assert!(matches!(error, ExecError::UnsupportedStringConstantValue));
    }
}
