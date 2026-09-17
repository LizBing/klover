use std::sync::Mutex;

use super::{
    arguments::Arguments,
    ms_api::{self, MsInitError},
};
use crate::gc_bindings::gc_bindings::gc_init;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmInitErrorKind {
    AlreadyInitialized,
    Metaspace(MsInitError),
    GcInitFailed,
}

#[derive(Debug)]
pub struct VmInitError {
    pub args: Arguments,
    pub kind: VmInitErrorKind,
}

pub type VmInitResult = Result<(), VmInitError>;

/// Initialize a single VM. Every failure returns ownership of the arguments.
/// Repeated calls are rejected, even with identical arguments. Native GC
/// failure leaves the VM unpublished; the initialized metaspace may be reused.
pub fn try_init(args: Arguments) -> VmInitResult {
    static INIT_LOCK: Mutex<()> = Mutex::new(());
    let _guard = INIT_LOCK.lock().expect("VM initialization panicked");

    if Arguments::is_initialized() {
        return Err(VmInitError {
            args,
            kind: VmInitErrorKind::AlreadyInitialized,
        });
    }
    if let Err(error) = ms_api::ensure_initialized() {
        return Err(VmInitError {
            args,
            kind: VmInitErrorKind::Metaspace(error),
        });
    }
    // SAFETY: INIT_LOCK serializes GC initialization; arguments are published
    // only after native initialization succeeds, preventing a second success.
    if !unsafe { gc_init(args.xmx) } {
        return Err(VmInitError {
            args,
            kind: VmInitErrorKind::GcInitFailed,
        });
    }
    Arguments::init(args);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_reinitialization_returns_arguments_without_changing_vm() {
        crate::runtime::test_support::init_vm();
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
}
