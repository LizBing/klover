mod test_harness;

use klover::engine::{outcome::RetValue, slot::Slot};
use test_harness::{expect_int, load_class, run};

fn expect_bool(value: i32) -> bool {
    match value {
        0 => false,
        1 => true,
        other => panic!("invalid boolean result: {other}"),
    }
}

fn run_bool(name: &str, descriptor: &str, args: Vec<Slot>) -> bool {
    let holder = load_class("ControlFlow");
    expect_bool(expect_int(run(&holder, name, descriptor, args)))
}

#[test]
fn loops_and_integer_stores() {
    let holder = load_class("ControlFlow");

    assert_eq!(
        expect_int(run(&holder, "sum", "(I)I", vec![Slot::int(0)])),
        0
    );
    assert_eq!(
        expect_int(run(&holder, "sum", "(I)I", vec![Slot::int(10)])),
        55
    );
    assert_eq!(
        expect_int(run(&holder, "countTo", "(I)I", vec![Slot::int(7)])),
        7
    );
    assert_eq!(
        expect_int(run(&holder, "factorial", "(I)I", vec![Slot::int(5)])),
        120
    );
    assert_eq!(
        expect_int(run(&holder, "firstEven", "(I)I", vec![Slot::int(7)])),
        8
    );
    assert_eq!(
        expect_int(run(
            &holder,
            "indexedStore",
            "(IIIII)I",
            vec![
                Slot::int(3),
                Slot::int(0),
                Slot::int(0),
                Slot::int(0),
                Slot::int(9)
            ],
        )),
        12
    );
}

#[test]
fn unary_integer_branches() {
    let cases = [
        ("isZero", 0, true),
        ("isZero", 1, false),
        ("isNonZero", 0, false),
        ("isNonZero", -1, true),
        ("isNegative", -1, true),
        ("isNegative", 0, false),
        ("isNonNegative", 0, true),
        ("isNonNegative", -1, false),
        ("isPositive", 1, true),
        ("isPositive", 0, false),
        ("isNonPositive", 0, true),
        ("isNonPositive", 1, false),
    ];

    for (name, value, expected) in cases {
        assert_eq!(run_bool(name, "(I)Z", vec![Slot::int(value)]), expected);
    }
}

#[test]
fn binary_integer_branches() {
    let cases = [
        ("intsEqual", 2, 2, true),
        ("intsEqual", 2, 3, false),
        ("intsNotEqual", 2, 3, true),
        ("intsNotEqual", 2, 2, false),
        ("intLessThan", 2, 3, true),
        ("intLessThan", 3, 2, false),
        ("intGreaterOrEqual", 3, 3, true),
        ("intGreaterOrEqual", 2, 3, false),
        ("intGreaterThan", 3, 2, true),
        ("intGreaterThan", 2, 2, false),
        ("intLessOrEqual", 2, 2, true),
        ("intLessOrEqual", 3, 2, false),
    ];

    for (name, lhs, rhs, expected) in cases {
        assert_eq!(
            run_bool(name, "(II)Z", vec![Slot::int(lhs), Slot::int(rhs)]),
            expected
        );
    }
}

#[test]
fn max_and_void_return() {
    let holder = load_class("ControlFlow");

    assert_eq!(
        expect_int(run(
            &holder,
            "max",
            "(II)I",
            vec![Slot::int(7), Slot::int(3)]
        )),
        7
    );
    assert_eq!(
        expect_int(run(
            &holder,
            "max",
            "(II)I",
            vec![Slot::int(3), Slot::int(7)]
        )),
        7
    );
    assert!(matches!(
        run(&holder, "doNothing", "()V", vec![]),
        klover::engine::outcome::ThreadExit::Returned(RetValue::Void)
    ));
}
