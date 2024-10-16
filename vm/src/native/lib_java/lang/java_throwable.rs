use anyhow::Result;

use crate::{
    data_types::{
        object::ObjRef,
        value::{RefType, Value},
    },
    native::registry::NativeMethod,
    vm::VM,
};

pub(crate) const FILL_IN_STACK_TRACE: &str = "fillInStackTrace(I)Ljava/lang/Throwable;";

pub fn jvm_fill_in_stack_trace(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    let mut this = this.unwrap();
    let stack_trace = vm.create_stack_trace().unwrap();
    this.set_field("stackTrace", Value::new_array(stack_trace));
    Ok(Some(Value::new_object(this)))
}
