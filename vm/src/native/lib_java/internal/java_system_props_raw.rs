use crate::{
    data_types::{reference_types::ObjRef, value::Value},
    vm::VM,
};
use anyhow::Result;

// const NATIVES: [(&str, NativeMethod); 1] = [(
//     "platformProperties()[Ljava/lang/String;",
//     jvm_platform_properties,
// )];

pub(crate) const PLATFORM_PROPS_SIG: &str = "platformProperties()[Ljava/lang/String;";

pub fn jvm_platform_properties(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}
