use crate::runtime::arguments::Arguments;

unsafe extern "C" {
    fn ms_init() -> bool;
    fn gc_init(xmx: usize);
}

pub fn vm_init(args: Arguments) {
    unsafe {
        assert!(ms_init(), "Failed to initialize metaspace.");
        gc_init(args.xmx);
    }

    Arguments::init(args);
}
