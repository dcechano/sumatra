use anyhow::Result;

use crate::{
    data_types::{reference_types::ObjRef, value::Value},
    vm::VM,
};

pub(crate) const GET_CLASS_SIG: &str = "getClass()Ljava/lang/Class;";

pub(crate) fn jvm_get_class(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    let class_obj = vm.get_class_obj(this.unwrap()).unwrap();
    Ok(Some(Value::new_object(class_obj)))
}
