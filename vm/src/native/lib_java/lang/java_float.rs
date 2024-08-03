use anyhow::{bail, Result};

use crate::{
    data_types::{object::ObjRef, value::Value},
    vm::VM,
};

pub(crate) const FLOAT_TO_RAW_INT_BITS_SIG: &str = "floatToRawIntBits(F)I";

pub fn jvm_float_to_raw_int_bits(
    _: &mut VM,
    _: Option<ObjRef>,
    args: Vec<Value>,
) -> Result<Option<Value>> {
    assert_eq!(args.len(), 1);
    let Value::Float(float) = args[0] else {
        bail!("Expected float as first arg in jvm_float_to_raw_int_bits");
    };

    Ok(Some(Value::Int(unsafe {
        std::mem::transmute::<f32, i32>(float)
    })))
}
