use sumatra_vm::{
    data_types::{object::ObjRef, value::Value},
    result::Result,
    vm::VM,
};

pub(crate) const IS_BIG_ENDIAN_SIG: &str = "isBigEndian()Z";

#[no_mangle]
pub fn JAVA_LANG_STRINGUTF16_is_big_endian(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    //TODO Do not hard code true!
    println!(
        "WARNING: Native JAVA_LANG_STRING_UTF16_is_big_endian is returning \
        true as a hardcoded value."
    );
    Ok(Some(Value::Int(1)))
}
