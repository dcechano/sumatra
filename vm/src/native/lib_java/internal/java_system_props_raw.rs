use crate::{
    data_types::{reference_types::ObjRef, value::Value},
    vm::VM,
};
use anyhow::Result;
use sumatra_parser::instruction::ArrayType;

// const NATIVES: [(&str, NativeMethod); 1] = [(
//     "platformProperties()[Ljava/lang/String;",
//     jvm_platform_properties,
// )];

pub(crate) const PLATFORM_PROPS_SIG: &str = "platformProperties()[Ljava/lang/String;";

pub fn jvm_platform_properties(
    vm: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    let mut string_array = vm.heap().new_array(2, ArrayType::Ref);
    let key = vm.create_java_string("_display_country_NDX", false);
    string_array.insert(0, Value::new_object(key));
    string_array.insert(1, Value::Null);
    Ok(Some(Value::new_array(string_array)))
}
