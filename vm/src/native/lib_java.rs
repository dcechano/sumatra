use crate::{
    native::{native_identifier::NativeIdentifier, registry::NativeMethod},
    vm::VM,
};

pub mod java_class;
pub mod java_object;
pub mod java_system;
pub(crate) const REGISTER_NATIVES_SIG: &str = "registerNatives()V";

pub const JAVA_LANG_OBJECT: &str = "java/lang/Object";
pub const JAVA_LANG_CLASS: &str = "java/lang/Class";
pub const JAVA_LANG_SYSTEM: &str = "java/lang/System";

pub(super) fn register_natives(vm: &mut VM, class: &str, natives: &[(&str, NativeMethod)]) {
    natives.iter().for_each(|(name, method)| {
        vm.native_registry.register(
            NativeIdentifier::new(class.to_string(), name.to_string()),
            *method,
        );
    });
}
