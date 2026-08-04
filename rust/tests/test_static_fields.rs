mod test_harness;

use klover::engine::slot::Slot;
use test_harness::{
    expect_double, expect_float, expect_int, expect_long, expect_ref, load_class, run,
};

#[test]
fn static_integer_family_round_trips() {
    let holder = load_class("StaticFields");

    assert_eq!(
        expect_int(run(&holder, "booleanRoundTrip", "(Z)I", vec![Slot::int(1)],)),
        1
    );
    assert_eq!(
        expect_int(run(&holder, "byteRoundTrip", "(B)I", vec![Slot::int(-5)],)),
        -5
    );
    assert_eq!(
        expect_int(run(
            &holder,
            "charRoundTrip",
            "(C)I",
            vec![Slot::int(65_530)],
        )),
        65_530
    );
    assert_eq!(
        expect_int(run(
            &holder,
            "shortRoundTrip",
            "(S)I",
            vec![Slot::int(-1_234)],
        )),
        -1_234
    );
    assert_eq!(
        expect_int(run(
            &holder,
            "intRoundTrip",
            "(I)I",
            vec![Slot::int(123_456)],
        )),
        123_456
    );
}

#[test]
fn static_numeric_category_two_round_trips() {
    let holder = load_class("StaticFields");

    assert_eq!(
        expect_long(run(
            &holder,
            "longRoundTrip",
            "(J)J",
            vec![
                Slot::long_high(0x1_2345_6789),
                Slot::long_low(0x1_2345_6789)
            ],
        )),
        0x1_2345_6789
    );

    let double = -1234.5;
    assert_eq!(
        expect_double(run(
            &holder,
            "doubleRoundTrip",
            "(D)D",
            vec![Slot::double_high(double), Slot::double_low(double)],
        )),
        double
    );
}

#[test]
fn static_float_and_default_reference_round_trip() {
    let holder = load_class("StaticFields");

    assert_eq!(
        expect_float(run(
            &holder,
            "floatRoundTrip",
            "(F)F",
            vec![Slot::float(3.25)],
        )),
        3.25
    );
    assert_eq!(
        expect_ref(run(
            &holder,
            "defaultReference",
            "()Ljava/lang/Object;",
            vec![]
        )),
        0
    );
}

#[test]
fn field_resolution_returns_the_declaring_superclass() {
    let holder = load_class("StaticFields");
    assert_eq!(
        expect_int(run(
            &holder,
            "inheritedRoundTrip",
            "(I)I",
            vec![Slot::int(77)],
        )),
        77
    );
}
