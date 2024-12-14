use anyhow::Result;

use sumatra_vm::{
    data_types::{object::ObjRef, value::Value},
    vm::VM,
};

pub(crate) const GET_CLASS_SIG: &str = "getClass()Ljava/lang/Class;";
pub(crate) const HASH_CODE_SIG: &str = "hashCode()I";

#[no_mangle]
pub(crate) fn JAVA_LANG_OBJECT_get_class(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    let class_obj = vm.get_class_obj(this.unwrap()).unwrap();
    Ok(Some(Value::new_object(class_obj)))
}

#[no_mangle]
pub(crate) fn JAVA_LANG_OBJECT_hash_code(
    _: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    Ok(Some(Value::Int(this.unwrap().hash_code())))
}
