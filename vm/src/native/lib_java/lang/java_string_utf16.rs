use anyhow::Result;

use crate::{
    data_types::{reference_types::ObjRef, value::Value},
    vm::VM,
};

pub(crate) const IS_BIG_ENDIAN_SIG: &str = "isBigEndian()Z";

pub fn jvm_is_big_endian(_: &mut VM, _: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    Ok(Some(Value::Int(1)))
}
