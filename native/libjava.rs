use crate::{
    native::{native_identifier::NativeIdentifier, registry::NativeMethod},
    vm::VM,
};

pub mod io;
pub mod lang;

pub(crate) const REGISTER_NATIVES_SIG: &str = "registerNatives()V";

pub const JAVA_LANG_OBJECT: &str = "java/lang/Object";
pub const JAVA_LANG_CLASS: &str = "java/lang/Class";
pub const JAVA_LANG_FLOAT: &str = "java/lang/Float";
pub const JAVA_LANG_DOUBLE: &str = "java/lang/Double";
pub const JAVA_LANG_RUNTIME: &str = "java/lang/Runtime";
pub const JAVA_LANG_SYSTEM: &str = "java/lang/System";
pub const JAVA_LANG_STRING_UTF16: &str = "java/lang/StringUTF16";
pub const JAVA_LANG_THROWABLE: &str = "java/lang/Throwable";

pub const JAVA_IO_FILE_INPUT_STREAM: &str = "java/io/FileInputStream";

pub(super) fn register_natives(vm: &mut VM, class: &str, natives: &[(&str, NativeMethod)]) {
    natives.iter().for_each(|(name, method)| {
        vm.native_registry.register(
            NativeIdentifier::new(class.to_string(), name.to_string()),
            *method,
        );
    });
}
