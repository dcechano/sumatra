use std::{fs, fs::File};
use sumatra_parser::class_file::ClassFile;
use sumatra_vm::class::Class;

const OBJECT_FILE: &'static str = "tests/classes/Object.class";

#[test]
fn get_name() {
    let object = parse_object_class();
    assert_eq!(object.get_name(), "java/lang/Object".to_string());
}

fn parse_object_class() -> Class { Class::from(ClassFile::parse_class(OBJECT_FILE).unwrap()) }
