use anyhow::{bail, Result};

use crate::{
    data_types::{
        object::ObjRef,
        value::{RefType, Value},
    },
    native::{
        lib_java::{register_natives, JAVA_LANG_SYSTEM},
        registry::NativeMethod,
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

pub(crate) fn jvm_open0(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

pub(crate) fn jvm_read0(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

pub(crate) fn jvm_read_bytes(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

pub(crate) fn jvm_length0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

pub(crate) fn jvm_position0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

pub(crate) fn jvm_skip0(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

pub(crate) fn jvm_available0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

pub(crate) fn jvm_is_regular_file0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

pub(crate) fn jvm_init_ids(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    println!(
        "[jvm_init_ids] java_file_input_stream.rs is not setting the ID \
        for FileInputStream.fd"
    );
    Ok(None)
}
