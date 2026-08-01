mod test_harness;

use klover::engine::{exec_error::ExecError, slot::Slot};
use test_harness::{expect_int, expect_long, load_class, run, try_run};

#[test]
fn invokestatic_passes_category_one_arguments_and_returns_int() {
    let holder = load_class("StaticCaller");
    let exit = run(
        &holder,
        "callInt",
        "(II)I",
        vec![Slot::int(3), Slot::int(4)],
    );

    assert_eq!(expect_int(exit), 14);
}

#[test]
fn invokestatic_passes_category_two_arguments_and_returns_long() {
    let holder = load_class("StaticCaller");
    let exit = run(
        &holder,
        "callLong",
        "(JJ)J",
        vec![
            Slot::long_high(40),
            Slot::long_low(40),
            Slot::long_high(2),
            Slot::long_low(2),
        ],
    );

    assert_eq!(expect_long(exit), 42);
}

#[test]
fn invokestatic_accepts_void_return() {
    let holder = load_class("StaticCaller");
    let exit = run(&holder, "callVoid", "(I)I", vec![Slot::int(7)]);

    assert_eq!(expect_int(exit), 8);
}

#[test]
fn invokestatic_does_not_skip_declared_class_initializer() {
    let holder = load_class("StaticCaller");
    let error = try_run(&holder, "callNeedsClinit", "()I", vec![]).unwrap_err();

    assert!(matches!(error, ExecError::ClassInitializerNotSupported));
}

#[test]
fn root_static_invocation_uses_the_same_initialization_path() {
    let holder = load_class("StaticNeedsClinit");

    for _ in 0..2 {
        let error = try_run(&holder, "value", "()I", vec![]).unwrap_err();
        assert!(matches!(error, ExecError::ClassInitializerNotSupported));
    }
}
