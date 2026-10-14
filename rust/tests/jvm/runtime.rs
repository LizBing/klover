//! VM lifecycle checks share the JVM integration suite's initialized runtime.
use klover::runtime::{
    arguments::Arguments,
    vm::{VmInitErrorKind, try_init},
};

#[test]
fn concurrent_reinitialization_returns_arguments_without_changing_vm() {
    crate::support::init_vm();
    let published = Arguments::get();
    let original_xmx = published.xmx;
    let original_path = published.bs_class_path.clone();
    std::thread::scope(|scope| {
        for i in 0..8 {
            scope.spawn(move || {
                let path = format!("unused-classpath-{i}");
                let xmx = (i + 1) * 1024 * 1024;
                let error = try_init(Arguments {
                    bs_class_path: path.clone(),
                    ms_pretouch: false,
                    xmx,
                })
                .expect_err("a live VM must reject reinitialization");
                assert_eq!(error.kind, VmInitErrorKind::AlreadyInitialized);
                assert_eq!(error.args.bs_class_path, path);
                assert_eq!(error.args.xmx, xmx);
            });
        }
    });
    assert_eq!(Arguments::get().xmx, original_xmx);
    assert_eq!(Arguments::get().bs_class_path, original_path);
    assert!(std::ptr::eq(Arguments::get(), published));
}
