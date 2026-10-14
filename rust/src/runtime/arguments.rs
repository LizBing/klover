use std::sync::OnceLock;

#[derive(Debug)]
pub struct Arguments {
    pub bs_class_path: String,
    pub ms_pretouch: bool,
    pub xmx: usize,
}

static ARGUMENTS: OnceLock<Arguments> = OnceLock::new();

impl Arguments {
    pub(super) fn init(args: Arguments) {
        ARGUMENTS.set(args).unwrap()
    }

    pub(super) fn is_initialized() -> bool {
        ARGUMENTS.get().is_some()
    }

    /// Allocator-only users can run before a VM publishes its arguments.
    pub(super) fn ms_pretouch() -> bool {
        ARGUMENTS.get().is_some_and(|args| args.ms_pretouch)
    }

    pub fn get() -> &'static Arguments {
        ARGUMENTS.get().unwrap()
    }
}
