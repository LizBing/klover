use super::support::{is_child, isolated};
use crate::runtime::{
    arguments::Arguments,
    ms_api::{self, MsAllocator, MsBox, MsInitError},
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
            let published = Arguments::get();
            for attempted in [
                arguments(4096),
                Arguments {
                    bs_class_path: "/different-test-path".into(),
                    ms_pretouch: false,
                    xmx: 8192,
                },
            ] {
                let expected_path = attempted.bs_class_path.clone();
                let expected_pretouch = attempted.ms_pretouch;
                let expected_xmx = attempted.xmx;
                let error = vm::try_init(attempted).unwrap_err();
                assert_eq!(error.kind, VmInitErrorKind::AlreadyInitialized);
                assert_eq!(error.args.bs_class_path, expected_path);
                assert_eq!(error.args.ms_pretouch, expected_pretouch);
                assert_eq!(error.args.xmx, expected_xmx);
                assert!(std::ptr::eq(Arguments::get(), published));
                assert_eq!(Arguments::get().bs_class_path, "/unused-test-path");
                assert!(Arguments::get().ms_pretouch);
                assert_eq!(Arguments::get().xmx, 4096);
            }
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
            assert!(error.args.ms_pretouch);
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
fn retry_after_failed_gc_initialization() {
    isolated(
        concat!(module_path!(), "::retry_after_failed_gc_initialization"),
        || {
            assert_eq!(
                vm::try_init(arguments(0)).unwrap_err().kind,
                VmInitErrorKind::GcInitFailed
            );
            assert!(
                std::panic::catch_unwind(Arguments::get).is_err(),
                "failed initialization published VM arguments"
            );
            // vm::try_init documents that this initialized metaspace may be reused.
            vm::try_init(Arguments {
                bs_class_path: "/retry-test-path".into(),
                ms_pretouch: false,
                xmx: 4096,
            })
            .expect("retry after failed GC initialization");
            let published = Arguments::get();
            assert_eq!(published.bs_class_path, "/retry-test-path");
            assert!(!published.ms_pretouch);
            assert_eq!(published.xmx, 4096);

            let error = vm::try_init(arguments(8192)).unwrap_err();
            assert_eq!(error.kind, VmInitErrorKind::AlreadyInitialized);
            assert_eq!(error.args.bs_class_path, "/unused-test-path");
            assert!(error.args.ms_pretouch);
            assert_eq!(error.args.xmx, 8192);
            assert!(std::ptr::eq(Arguments::get(), published));
        },
    );
}

#[test]
fn independently_initialized_metaspace_is_reused_by_vm() {
    isolated(
        concat!(
            module_path!(),
            "::independently_initialized_metaspace_is_reused_by_vm"
        ),
        || {
            ms_api::initialize().expect("initialize metaspace before starting a VM");
            let allocator = MsAllocator::new();
            let before_vm = MsBox::new(&allocator, 0x1234_5678_u64);
            assert_eq!(ms_api::initialize(), Err(MsInitError::AlreadyInitialized));
            assert!(
                std::panic::catch_unwind(Arguments::get).is_err(),
                "metaspace initialization published VM arguments"
            );

            vm::try_init(arguments(4096)).expect("reuse independently initialized metaspace");
            assert_eq!(Arguments::get().bs_class_path, "/unused-test-path");
            assert!(Arguments::get().ms_pretouch);
            assert_eq!(Arguments::get().xmx, 4096);

            // Existing metadata must survive VM startup and subsequent allocation.
            let after_vm = MsBox::new(&allocator, 0x8765_4321_u64);
            assert!(!std::ptr::eq(&*before_vm, &*after_vm));
            assert_eq!(*before_vm, 0x1234_5678);
            assert_eq!(*after_vm, 0x8765_4321);
        },
    );
}
