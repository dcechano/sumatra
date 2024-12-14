#![allow(warnings)] // TODO Remove
use sumatra_vm::vm::VM;

const CLASS_PATH: &str =
    "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/";

const JDK: &str = "/home/dylan/Documents/RustProjects/sumatra/jdk/compiled/java.base/";

const CLASSES: [&str; 5] = [
    "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/Main.class",
    "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/Interface.class",
    "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/Import.class",
    "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/Simple.class",
    "/home/dylan/Documents/RustProjects/sumatra/parser/tests/rt/java/lang/System.class",
];

fn main() {
    env_logger::init();

    let mut vm = VM::init(JDK.into(), CLASS_PATH.into());
    vm.run("Main.class").unwrap()
}
