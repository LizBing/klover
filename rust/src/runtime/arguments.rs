use std::sync::OnceLock;

#[derive(Debug)]
pub struct Arguments {
    pub bs_class_path: String,
    pub xmx: usize
}

static ARGUMENTS: OnceLock<Arguments> = OnceLock::new();

impl Arguments {
    pub fn init(args: Arguments) {
        ARGUMENTS.set(args).unwrap()
    }
    
    pub fn get() -> &'static Arguments {
        ARGUMENTS.get().unwrap()
    }
}
