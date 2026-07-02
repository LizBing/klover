use std::fs;

use crate::runtime::arguments::Arguments;

pub struct ClassPath;

impl ClassPath {
    pub fn read_bs_class(name: &str) -> Option<Vec<u8>> {
        let path = format!("{}/{}.class", Arguments::get().bs_class_path, name);
        
        match fs::read(path) {
            Ok(x) => Some(x),
            Err(_) => None
        }
    }
}
