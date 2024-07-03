use anyhow::Result;

use crate::{
    data_types::{object::ObjRef, value::Value},
    native::{
        lib_java::{register_natives, JAVA_LANG_SYSTEM},
        registry::NativeMethod,
    },
    vm::VM,
};

const NATIVES: [(&str, NativeMethod); 8] = [
    ("setIn0(Ljava/io/InputStream;)V", jvm_set_in0),
    ("setIn0(Ljava/io/PrintStream;)V", jvm_set_out0),
    ("setErr0(Ljava/io/PrintStream;)V", jvm_set_err0),
    ("currentTimeMillis()J", jvm_current_time_millis),
    ("nanoTime()J", jvm_nano_time),
    (
        "arraycopy(Ljava/lang/Object;ILjava/lang/Object;II)V",
        jvm_arraycopy,
    ),
    (
        "identityHashCode(Ljava/lang/Object;()I",
        jvm_identity_hash_code,
    ),
    (
        "mapLibraryName(Ljava/lang/String;)Ljava/lang/String;",
        jvm_map_library_name,
    ),
];

pub fn jvm_register_natives(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    register_natives(vm, JAVA_LANG_SYSTEM, &NATIVES);
    Ok(None)
}

fn jvm_set_in0(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_set_out0(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_set_err0(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_current_time_millis(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_nano_time(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_arraycopy(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_identity_hash_code(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_map_library_name(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}
