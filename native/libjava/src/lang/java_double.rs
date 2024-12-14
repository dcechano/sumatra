use anyhow::{bail, Result};
use sumatra_vm::{
    data_types::{object::ObjRef, value::Value},
    vm::VM,
};

pub(crate) const DOUBLE_TO_RAW_LONG_BITS_SIG: &str = "doubleToRawLongBits(D)J";
pub(crate) const LONG_BITS_TO_DOUBLE_SIG: &str = "longBitsToDouble(J)D";

#[allow(clippy::transmute_float_to_int)]
#[no_mangle]
pub fn JAVA_LANG_DOUBLE_double_to_raw_long_bits(
    _: &mut VM,
    _: Option<ObjRef>,
    args: Vec<Value>,
) -> Result<Option<Value>> {
    // args.len is 2 because doubles and longs take 2 spots
    assert_eq!(args.len(), 2);
    let Value::Double(double) = args[0] else {
        bail!("Expected double as first arg in JAVA_LANG_DOUBLE_double_to_raw_long_bits");
    };

    Ok(Some(Value::Long(unsafe {
        std::mem::transmute::<f64, i64>(double)
    })))
}

#[allow(clippy::transmute_int_to_float)]
#[no_mangle]
pub fn JAVA_LANG_DOUBLE_long_bits_to_double(
    _: &mut VM,
    _: Option<ObjRef>,
    args: Vec<Value>,
) -> Result<Option<Value>> {
    // args.len is 2 because doubles and longs take 2 spots
    assert_eq!(args.len(), 2);
    let Value::Long(long) = args[0] else {
        bail!("Expected long as first arg in JAVA_LANG_DOUBLE_long_bits_to_double");
    };

    Ok(Some(Value::Double(unsafe {
        std::mem::transmute::<i64, f64>(long)
    })))
}
