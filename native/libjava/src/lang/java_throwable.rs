use sumatra_vm::{
    data_types::{
        object::ObjRef,
        value::{RefType, Value},
    },
    result::Result,
    vm::VM,
};

pub(crate) const FILL_IN_STACK_TRACE: &str = "fillInStackTrace(I)Ljava/lang/Throwable;";

#[no_mangle]
pub fn JAVA_LANG_THROWABLE_fill_in_stack_trace(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    let mut this = this.unwrap();
    let stack_trace = vm.create_stack_trace().unwrap();
    this.set_field("stackTrace", Value::new_array(stack_trace));
    Ok(Some(Value::new_object(this)))
}
