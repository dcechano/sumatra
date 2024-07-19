use anyhow::Result;

use sumatra_parser::instruction::ArrayType;

use crate::{
    data_types::{
        array::ArrayComp,
        object::ObjRef,
        value::{RefType::Array, Value},
    },
    vm::VM,
};

// The number of static fields in jdk.internal.util.SystemProps.Raw.java
// https://github.com/openjdk/jdk/blob/jdk-21%2B35/src/java.base/share/classes/jdk/internal/util/SystemProps.java#L214
const NUM_PLATFORM_PROPS: usize = 40;

pub(crate) const PLATFORM_PROPS_SIG: &str = "platformProperties()[Ljava/lang/String;";
pub(crate) const VM_PROPS_SIG: &str = "vmProperties()[Ljava/lang/String;";

const STRING_CLASS: &str = "java/lang/String";

pub fn jvm_platform_properties(
    vm: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    let mut string_array = vm.heap().new_array(
        NUM_PLATFORM_PROPS,
        ArrayComp::Class(STRING_CLASS.to_string()),
    );
    (0..NUM_PLATFORM_PROPS).for_each(|index| {
        string_array.insert(index, Value::Null);
    });
    Ok(Some(Value::new_array(string_array)))
}

pub fn jvm_vm_properties(vm: &mut VM, _: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    let mut string_array = vm
        .heap()
        .new_array(2, ArrayComp::Class(STRING_CLASS.to_string()));
    string_array.insert(0, Value::Null);
    string_array.insert(1, Value::Null);
    Ok(Some(Value::new_array(string_array)))
}
