mod test_harness;

use klover::engine::slot::Slot;
use test_harness::{expect_ref, load_class, run};

#[test]
fn fixed_and_indexed_reference_loads() {
    let holder = load_class("ReferenceLoads");
    let null = Slot::reference(0);

    for (name, descriptor, args) in [
        (
            "load0",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            vec![null],
        ),
        (
            "load1",
            "(ILjava/lang/Object;)Ljava/lang/Object;",
            vec![Slot::int(1), null],
        ),
        (
            "load2",
            "(IILjava/lang/Object;)Ljava/lang/Object;",
            vec![Slot::int(1), Slot::int(2), null],
        ),
        (
            "load3",
            "(IIILjava/lang/Object;)Ljava/lang/Object;",
            vec![Slot::int(1), Slot::int(2), Slot::int(3), null],
        ),
        (
            "loadIndexed",
            "(IIIILjava/lang/Object;)Ljava/lang/Object;",
            vec![Slot::int(1), Slot::int(2), Slot::int(3), Slot::int(4), null],
        ),
    ] {
        assert_eq!(expect_ref(run(&holder, name, descriptor, args)), 0);
    }
}

#[test]
fn reference_store_load_round_trip() {
    let holder = load_class("ReferenceLoads");
    assert_eq!(
        expect_ref(run(
            &holder,
            "roundTrip",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            vec![Slot::reference(0)],
        )),
        0
    );
}
