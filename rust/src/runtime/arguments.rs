use std::sync::OnceLock;

#[derive(Debug)]
pub struct Arguments {
    pub boot_class_path: String,
}

pub static ARGUMENTS: OnceLock<Arguments> = OnceLock::new();

pub fn init_arguments(args: Arguments) {
    ARGUMENTS.set(args).unwrap()
}
