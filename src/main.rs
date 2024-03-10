use crate::class_file::ClassFile;

mod access_flags;
mod attribute;
mod class_file;
mod constant;
mod exception;
mod field;
mod instruction;
mod method;
mod stack_map;

fn main() {
    let class_file = ClassFile::parse_class(
        "/home/dylan/Documents/RustProjects/sumatra/java/target/production/java/Simple.class",
    )
    .unwrap();

    println!("=========== Parsed .class file ===========");
    println!("{:?}", class_file);
}
