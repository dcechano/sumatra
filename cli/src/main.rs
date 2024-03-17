use sumatra_parser::class_file::ClassFile;

fn main() {
    let class_file = ClassFile::parse_class(
        "/home/dylan/Documents/RustProjects/sumatra/java/target/production/java/Main.class",
    )
    .unwrap();

    println!("=========== Parsed .class file ===========");
    println!("{:#?}", class_file);
}
