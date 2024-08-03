use crate::{
    data_types::{object::ObjRef, value::Value},
    vm::VM,
};
use anyhow::{bail, Result};

pub(crate) const DOUBLE_TO_RAW_LONG_BITS_SIG: &str = "doubleToRawLongBits(D)J";

pub fn jvm_double_to_raw_long_bits(
    _: &mut VM,
    _: Option<ObjRef>,
    args: Vec<Value>,
) -> Result<Option<Value>> {
    // args.len is 2 because doubles and longs take 2 spots
    assert_eq!(args.len(), 2);
    let Value::Double(double) = args[0] else {
        bail!("Expected double as first arg in jvm_double_to_raw_long_bits");
    };

    Ok(Some(Value::Long(unsafe {
        std::mem::transmute::<f64, i64>(double)
    })))
}
