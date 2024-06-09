use crate::{
    data_types::{reference_types::ObjRef, value::Value},
    native::{
        lib_java::JAVA_LANG_CLASS, native_identifier::NativeIdentifier, registry::NativeMethod,
    },
    vm::VM,
};

const NATIVES: [(&str, NativeMethod); 1] = [
    ("platformProperties"), /* (
                             *     "forName0(Ljava/lang/String;ZLjava/lang/ClassLoader;Ljava/
                             * lang/Class)Ljava/lang/Class;",
                             *     crate::native::lib_java::lang::java_class::jvm_for_name0,
                             * ),
                             * ("isInstance(Ljava/lang/Object;)Z",
                             * crate::native::lib_java::lang::java_class::jvm_is_instance),
                             * (
                             *     "desiredAssertionStatus0(Ljava/lang/Class;)Z",
                             *     crate::native::lib_java::lang::java_class::jvm_desired_assertion_status0,
                             * ), */
];

pub fn jvm_register_natives(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> anyhow::Result<Option<Value>> {
    NATIVES.iter().for_each(|(name, method)| {
        vm.native_registry.register(
            NativeIdentifier::new(JAVA_LANG_CLASS.to_string(), name.to_string()),
            *method,
        );
    });

    Ok(None)
}
