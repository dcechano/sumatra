use crate::{
    native::{native_identifier::NativeIdentifier, registry::NativeMethod},
    vm::VM,
};

pub mod internal;
pub mod lang;

pub(crate) const REGISTER_NATIVES_SIG: &str = "registerNatives()V";

pub const JAVA_LANG_OBJECT: &str = "java/lang/Object";
pub const JAVA_LANG_CLASS: &str = "java/lang/Class";
pub const JAVA_LANG_FLOAT: &str = "java/lang/Float";
pub const JAVA_LANG_DOUBLE: &str = "java/lang/Double";
pub const JAVA_LANG_SYSTEM: &str = "java/lang/System";
pub const JAVA_LANG_STRING_UTF16: &str = "java/lang/StringUTF16";

pub const JDK_INTERNAL_SYSTEM_PROPS_RAW: &str = "jdk/internal/util/SystemProps$Raw";
pub const JDK_INTERNAL_MISC_UNSAFE: &str = "jdk/internal/misc/Unsafe";

pub(super) fn register_natives(vm: &mut VM, class: &str, natives: &[(&str, NativeMethod)]) {
    natives.iter().for_each(|(name, method)| {
        vm.native_registry.register(
            NativeIdentifier::new(class.to_string(), name.to_string()),
            *method,
        );
    });
}
