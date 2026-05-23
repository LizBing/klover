use std::{fs::File, io::{self, Read}};

use rust::class_parser::class_file;

#[test]
fn foo() {
    let mut f = File::open("../test_data/classes/HelloKlover.class").unwrap();

    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();

    let builder = class_file::ClassFileBuilder::from(&buf).unwrap();
    println!("{:?}", &builder);
}

