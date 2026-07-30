mod test_harness;

use klover::engine::{
    outcome::{RetValue, ThreadExit},
    slot::Slot,
};
use test_harness::{expect_double, expect_float, expect_int, expect_long, load_class, run};

fn int_args(count: usize) -> Vec<Slot> {
    (1..=count).map(|value| Slot::int(value as i32)).collect()
}

#[test]
fn integer_stores() {
    let holder = load_class("StoreOps");
    let cases = [
        ("int0", "()I", 0, 10),
        ("int1", "(I)I", 1, 2),
        ("int2", "(II)I", 2, 3),
        ("int3", "(III)I", 3, 6),
        ("intIndexed", "(IIII)I", 4, 5),
    ];

    for (name, descriptor, arg_count, expected) in cases {
        assert_eq!(
            expect_int(run(&holder, name, descriptor, int_args(arg_count))),
            expected
        );
    }
}

#[test]
fn long_stores() {
    let holder = load_class("StoreOps");

    for (name, descriptor, arg_count, expected) in [
        ("long0", "()J", 0, 10),
        ("long1", "(I)J", 1, 11),
        ("long2", "(II)J", 2, 12),
        ("long3", "(III)J", 3, 13),
        ("longIndexed", "(IIII)J", 4, 14),
    ] {
        assert_eq!(
            expect_long(run(&holder, name, descriptor, int_args(arg_count))),
            expected
        );
    }
}

#[test]
fn floating_point_stores() {
    let holder = load_class("StoreOps");

    for (name, descriptor, arg_count, expected) in [
        ("float0", "()F", 0, 10.5),
        ("float1", "(I)F", 1, 11.5),
        ("float2", "(II)F", 2, 12.5),
        ("float3", "(III)F", 3, 13.5),
        ("floatIndexed", "(IIII)F", 4, 14.5),
    ] {
        assert_eq!(
            expect_float(run(&holder, name, descriptor, int_args(arg_count))),
            expected
        );
    }

    for (name, descriptor, arg_count, expected) in [
        ("double0", "()D", 0, 10.5),
        ("double1", "(I)D", 1, 11.5),
        ("double2", "(II)D", 2, 12.5),
        ("double3", "(III)D", 3, 13.5),
        ("doubleIndexed", "(IIII)D", 4, 14.5),
    ] {
        assert_eq!(
            expect_double(run(&holder, name, descriptor, int_args(arg_count))),
            expected
        );
    }
}

#[test]
fn reference_stores() {
    let holder = load_class("StoreOps");

    for (name, descriptor, arg_count) in [
        ("ref0", "()V", 0),
        ("ref1", "(I)V", 1),
        ("ref2", "(II)V", 2),
        ("ref3", "(III)V", 3),
        ("refIndexed", "(IIII)V", 4),
    ] {
        assert!(matches!(
            run(&holder, name, descriptor, int_args(arg_count)),
            ThreadExit::Returned(RetValue::Void)
        ));
    }
}
