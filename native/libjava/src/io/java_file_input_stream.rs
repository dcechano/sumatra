use anyhow::{bail, Result};

use sumatra_vm::{
    data_types::{
        object::ObjRef,
        value::{RefType, Value},
    },
    vm::VM,
};

pub(crate) const OPEN_0_SIG: &str = "open0(Ljava/lang/String;)V";
pub(crate) const READ_0_SIG: &str = "read0()I";
pub(crate) const READ_BYTES_SIG: &str = "readBytes([BII)I";
pub(crate) const LENGTH_0_SIG: &str = "length0()J";
pub(crate) const POSITION_0_SIG: &str = "position0()J";
pub(crate) const SKIP_0_SIG: &str = "skip0()J";
pub(crate) const AVAILABLE_0_SIG: &str = "available0()I";
pub(crate) const IS_REGULAR_FILE_0_SIG: &str = "isRegularFile0(Ljava/io/FileDescriptor;)Z";
pub(crate) const INIT_IDS_SIG: &str = "initIDs()V";

#[no_mangle]
pub(crate) fn JAVA_IO_FILEINPUTSTREAM_open0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub(crate) fn JAVA_IO_FILEINPUTSTREAM_read0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub(crate) fn JAVA_IO_FILEINPUTSTREAM_read_bytes(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub(crate) fn JAVA_IO_FILEINPUTSTREAM_length0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub(crate) fn JAVA_IO_FILEINPUTSTREAM_position0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub(crate) fn JAVA_IO_FILEINPUTSTREAM_skip0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub(crate) fn JAVA_IO_FILEINPUTSTREAM_available0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub(crate) fn JAVA_IO_FILEINPUTSTREAM_is_regular_file0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub(crate) fn JAVA_IO_FILEINPUTSTREAM_init_i_ds(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    println!(
        "[JAVA_IO_FILEINPUTSTREAM_init_ids] java_file_input_stream.rs is not setting the ID \
        for FileInputStream.fd"
    );
    Ok(None)
}
