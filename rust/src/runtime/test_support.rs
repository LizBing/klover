//! Shared test VM with one fixed configuration per test process.
use super::{arguments::Arguments, vm};
use std::sync::Once;

pub fn init_vm() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        vm::try_init(Arguments {
            bs_class_path: std::env::var("KLOVER_TEST_CLASSES").unwrap_or_else(|_| {
                concat!(env!("CARGO_MANIFEST_DIR"), "/../build/test-classes").into()
            }),
            xmx: 64 * 1024 * 1024,
        })
        .expect("initialize test VM");
    });
}
