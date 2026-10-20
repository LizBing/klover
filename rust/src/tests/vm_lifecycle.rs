use super::support::{is_child, isolated};
use crate::runtime::{
    arguments::Arguments,
    vm::{self, VmInitErrorKind},
};

fn arguments(xmx: usize) -> Arguments {
    Arguments {
        bs_class_path: "/unused-test-path".into(),
        ms_pretouch: true,
        xmx,
    }
}

#[test]
fn successful_init_publishes_arguments_and_rejects_reinitialization() {
    isolated(
        concat!(
            module_path!(),
            "::successful_init_publishes_arguments_and_rejects_reinitialization"
        ),
        || {
            vm::try_init(arguments(4096)).unwrap();
            assert_eq!(Arguments::get().xmx, 4096);
            assert!(Arguments::get().ms_pretouch);
            let error = vm::try_init(arguments(8192)).unwrap_err();
            assert_eq!(error.kind, VmInitErrorKind::AlreadyInitialized);
            assert_eq!(error.args.xmx, 8192);
            assert_eq!(Arguments::get().xmx, 4096);
        },
    );
}

#[test]
fn failed_gc_init_returns_arguments_without_publishing_vm() {
    isolated(
        concat!(
            module_path!(),
            "::failed_gc_init_returns_arguments_without_publishing_vm"
        ),
        || {
            let error = vm::try_init(arguments(0)).unwrap_err();
            assert_eq!(error.kind, VmInitErrorKind::GcInitFailed);
            assert_eq!(error.args.xmx, 0);
            assert_eq!(error.args.bs_class_path, "/unused-test-path");
            assert!(
                std::panic::catch_unwind(Arguments::get).is_err(),
                "failed initialization published VM arguments"
            );
        },
    );
}

#[test]
fn concurrent_initialization_has_exactly_one_winner() {
    isolated(
        concat!(
            module_path!(),
            "::concurrent_initialization_has_exactly_one_winner"
        ),
        || {
            let barrier = std::sync::Barrier::new(4);
            std::thread::scope(|scope| {
                let handles: Vec<_> = (0..4)
                    .map(|_| {
                        scope.spawn(|| {
                            barrier.wait();
                            vm::try_init(arguments(4096))
                        })
                    })
                    .collect();
                let mut winners = 0;
                for handle in handles {
                    match handle.join().unwrap() {
                        Ok(()) => winners += 1,
                        Err(error) => assert_eq!(error.kind, VmInitErrorKind::AlreadyInitialized),
                    }
                }
                assert_eq!(winners, 1);
            });
        },
    );
}

#[test]
fn failed_child_is_reported_by_isolation_helper() {
    // Exercise failure propagation; a green parent must not hide a red child.
    let name = concat!(
        module_path!(),
        "::failed_child_is_reported_by_isolation_helper"
    );
    if is_child(name) {
        panic!("intentional child failure");
    }
    let failure = std::panic::catch_unwind(|| {
        isolated(name, || {});
    });
    assert!(failure.is_err());
}

#[test]
fn empty_child_selection_is_rejected() {
    assert!(std::panic::catch_unwind(|| isolated("not_a_registered_test", || {})).is_err());
}

#[test]
#[ignore = "known runtime issue: retry rejects the metaspace left by failed GC initialization; see TESTING.md"]
fn retry_after_failed_gc_initialization() {
    isolated(
        concat!(module_path!(), "::retry_after_failed_gc_initialization"),
        || {
            assert_eq!(
                vm::try_init(arguments(0)).unwrap_err().kind,
                VmInitErrorKind::GcInitFailed
            );
            // vm::try_init documents that this initialized metaspace may be reused.
            vm::try_init(arguments(4096)).expect("retry after failed GC initialization");
        },
    );
}
