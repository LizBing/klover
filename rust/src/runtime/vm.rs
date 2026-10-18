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
    _private: (),
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
            _private: (),
            args,
            kind: VmInitErrorKind::AlreadyInitialized,
        });
    }
    if let Err(error) = ms_api::initialize() {
        return Err(VmInitError {
            _private: (),
            args,
            kind: VmInitErrorKind::Metaspace(error),
        });
    }
    // SAFETY: INIT_LOCK serializes GC initialization; arguments are published
    // only after native initialization succeeds, preventing a second success.
    if !unsafe { gc_init(args.xmx) } {
        return Err(VmInitError {
            _private: (),
            args,
            kind: VmInitErrorKind::GcInitFailed,
        });
    }
    Arguments::init(args);
    Ok(())
}
