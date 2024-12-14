use anyhow::{bail, Result};

use sumatra_vm::{
    data_types::{object::ObjRef, value::Value},
    vm::VM,
};

pub(crate) const FLOAT_TO_RAW_INT_BITS_SIG: &str = "floatToRawIntBits(F)I";

#[allow(clippy::transmute_float_to_int)]
#[no_mangle]
pub fn JAVA_LANG_FLOAT_float_to_raw_int_bits(
    _: &mut VM,
    _: Option<ObjRef>,
    args: Vec<Value>,
) -> Result<Option<Value>> {
    assert_eq!(args.len(), 1);
    let Value::Float(float) = args[0] else {
        bail!("Expected float as first arg in JAVA_LANG_FLOAT_float_to_raw_int_bits");
    };

    Ok(Some(Value::Int(unsafe {
        std::mem::transmute::<f32, i32>(float)
    })))
}
