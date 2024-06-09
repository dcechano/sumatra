use crate::{
    data_types::{reference_types::ObjRef, value::Value},
    native::{
        lib_java::JAVA_LANG_CLASS, native_identifier::NativeIdentifier, registry::NativeMethod,
    },
    vm::VM,
};
use anyhow::Result;

const NATIVES: [(&str, NativeMethod); 1] = [(
    "platformProperties()[Ljava/lang/String;",
    jvm_platform_properties,
)];

pub fn jvm_register_natives(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    NATIVES.iter().for_each(|(name, method)| {
        vm.native_registry.register(
            NativeIdentifier::new(JAVA_LANG_CLASS.to_string(), name.to_string()),
            *method,
        );
    });

    Ok(None)
}

pub fn jvm_platform_properties(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}
