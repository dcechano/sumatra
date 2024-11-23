mod util;
mod vm;

pub use util::*;
pub use vm::*;

const MAIN: &str = "main([Ljava/lang/String;)V";
const CLINIT: &str = "<clinit>()V";
const INIT: &str = "<init>()V";
pub(crate) const OBJECT: &str = "java/lang/Object";
pub(crate) const CLASS: &str = "java/lang/Class";
pub(crate) const SYSTEM: &str = "java/lang/System";
pub(crate) const STRING: &str = "java/lang/String";

const NUM_PRIMS: usize = 8;
pub const OBJECT_CLASS_ID: usize = NUM_PRIMS + 0;
pub const SYSTEM_CLASS_ID: usize = NUM_PRIMS + 1;
pub const CLASS_CLASS_ID: usize = NUM_PRIMS + 2;

const DEFAULT_VEC_SIZE: usize = 128;
