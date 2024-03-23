use sumatra_parser::class_file::ClassFile;

const CLASSES: [&str; 5] = [
    "/home/dylan/Documents/RustProjects/sumatra/java/target/production/java/Main.class",
    "/home/dylan/Documents/RustProjects/sumatra/java/target/production/java/Interface.class",
    "/home/dylan/Documents/RustProjects/sumatra/java/target/production/java/Import.class",
    "/home/dylan/Documents/RustProjects/sumatra/java/target/production/java/Simple.class",
    "/home/dylan/Documents/RustProjects/sumatra/java/target/production/java/Taco.class",
];
fn main() {
    // let class_file = ClassFile::parse_class(
    //     "/home/dylan/Documents/RustProjects/sumatra/java/target/production/java/
    // Simple.class", )
    // .unwrap();
    // println!("{:#?}", class_file);

    for class in CLASSES {
        println!();
        println!("Parsing: {class}");
        println!();
        println!("{:#?}", ClassFile::parse_class(class).unwrap());
    }
}
