#![allow(warnings)] // TODO Remove

use sumatra_parser::class_file::ClassFile;
use sumatra_vm::{class::Class, vm::VM};

const CLASS_PATH: &str =
    "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/";

const CLASSES: [&str; 5] = [
    "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/Main.class",
    "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/Interface.class",
    "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/Import.class",
    "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/Simple.class",
    "/home/dylan/Documents/RustProjects/sumatra/parser/tests/rt/java/lang/System.class",
];
fn main() {
    let class = CLASSES[0];
    let class_file = ClassFile::parse_class(class).unwrap();
    // println!("Running {:#?}", class);
    // println!("class {:#?}", class_file);
    let mut vm = VM::init(CLASS_PATH.into());
    vm.run("Simple.class").unwrap()

    // for class in CLASSES {
    //     println!();
    //     println!("Parsing: {class}");
    //     println!();
    //     println!("{:#?}", ClassFile::parse_class(class).unwrap());
    // }
}
