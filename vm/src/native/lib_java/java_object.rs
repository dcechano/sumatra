use anyhow::Result;

use crate::{
    data_types::{reference_types::ObjRef, value::Value},
    native::{lib_java::JAVA_LANG_OBJECT, native_identifier::NativeIdentifier},
    vm::VM,
};

pub fn jvm_register_natives(
    vm: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    let native_identifier = NativeIdentifier::new(
        JAVA_LANG_OBJECT.to_string(),
        "getClass()Ljava/lang/Class".to_string(),
    );
    vm.native_registry
        .register(native_identifier, jvm_get_class);
    Ok(None)
}

fn jvm_get_class(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    let class_obj = vm.get_class_obj(this.unwrap()).unwrap();
    Ok(Some(Value::new_object(class_obj)))
}
