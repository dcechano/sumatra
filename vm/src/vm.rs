mod util;

#[allow(clippy::module_inception)]
mod vm;

pub use vm::*;

const MAIN: &str = "main([Ljava/lang/String;)V";
const CLINIT: &str = "<clinit>()V";
#[allow(dead_code)]
const INIT: &str = "<init>()V";
pub(crate) const OBJECT: &str = "java/lang/Object";
pub(crate) const CLASS: &str = "java/lang/Class";
pub(crate) const SYSTEM: &str = "java/lang/System";
pub(crate) const STRING: &str = "java/lang/String";

const NUM_PRIMS: usize = 8;
#[allow(clippy::identity_op)]
pub const OBJECT_CLASS_ID: usize = NUM_PRIMS + 0;
pub const SYSTEM_CLASS_ID: usize = NUM_PRIMS + 1;
pub const CLASS_CLASS_ID: usize = NUM_PRIMS + 2;

const DEFAULT_VEC_SIZE: usize = 128;

impl Drop for VM {
    fn drop(&mut self) {
        if !self.frames.is_empty() {
            println!("============== PRINTING STACKFRAMES ==============");
            for (num, frame) in self.frames.iter().enumerate() {
                let pc = frame.pc;
                println!(
                    "[frame {num}] {}::{}{} - {:?}",
                    frame.class.get_name(),
                    frame.method.name,
                    frame.method.descriptor,
                    frame.method.code.op_code.get(pc).unwrap()
                );
            }
        }
    }
}
