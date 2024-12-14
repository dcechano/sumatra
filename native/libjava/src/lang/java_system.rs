use sumatra_vm::{
    data_types::{
        object::ObjRef,
        value::{RefType, Value},
    },
    native::NativeMethod,
    result::Result,
    vm::VM,
    vm_error,
};

const NATIVES: [(&str, NativeMethod); 9] = [
    ("setIn0(Ljava/io/InputStream;)V", JAVA_LANG_SYSTEM_set_in0),
    ("setIn0(Ljava/io/PrintStream;)V", JAVA_LANG_SYSTEM_set_out0),
    ("setErr0(Ljava/io/PrintStream;)V", JAVA_LANG_SYSTEM_set_err0),
    ("currentTimeMillis()J", JAVA_LANG_SYSTEM_current_time_millis),
    ("nanoTime()J", JAVA_LANG_SYSTEM_nano_time),
    (
        "arraycopy(Ljava/lang/Object;ILjava/lang/Object;II)V",
        JAVA_LANG_SYSTEM_arraycopy,
    ),
    (
        "identityHashCode(Ljava/lang/Object;()I",
        JAVA_LANG_SYSTEM_identity_hash_code,
    ),
    (
        "mapLibraryName(Ljava/lang/String;)Ljava/lang/String;",
        JAVA_LANG_SYSTEM_map_library_name,
    ),
    (
        "arraycopy(Ljava/lang/Object;ILjava/lang/Object;II)V",
        JAVA_LANG_SYSTEM_arraycopy,
    ),
];

#[no_mangle]
pub fn JAVA_LANG_SYSTEM_register_natives(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    Ok(None)
}

#[no_mangle]
fn JAVA_LANG_SYSTEM_set_in0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_SYSTEM_set_out0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_SYSTEM_set_err0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_SYSTEM_current_time_millis(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_SYSTEM_nano_time(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_SYSTEM_arraycopy(
    _: &mut VM,
    _: Option<ObjRef>,
    args: Vec<Value>,
) -> Result<Option<Value>> {
    assert_eq!(args.len(), 5);
    let Value::Ref(RefType::Array(mut src)) = args[0] else {
        vm_error!("Expected array as first arg in JAVA_LANG_SYSTEM_arraycopy");
    };
    let Value::Int(src_pos) = args[1] else {
        vm_error!("Expected int for second arg in JAVA_LANG_SYSTEM_arraycopy");
    };
    let Value::Ref(RefType::Array(mut dest)) = args[2] else {
        vm_error!("Expected array as third arg in JAVA_LANG_SYSTEM_arraycopy");
    };
    let Value::Int(dest_pos) = args[3] else {
        vm_error!("Expected int for fourth arg in JAVA_LANG_SYSTEM_arraycopy");
    };
    let Value::Int(length) = args[4] else {
        vm_error!("Expected int for fifth arg in JAVA_LANG_SYSTEM_arraycopy");
    };

    if src_pos < 0 || dest_pos < 0 {
        todo!("Throw IndexOutOfBoundsException")
    } else if src_pos + length > src.len() as i32 {
        todo!("Throw IndexOutOfBoundsException")
    } else if dest_pos + length > dest.len() as i32 {
        todo!("Throw IndexOutOfBoundsException")
    }

    // Component type checking is done in the insert method of the dest array
    let (src_pos, dest_pos, length) = (src_pos as usize, dest_pos as usize, length as usize);
    (0..length).for_each(|i| dest.insert(dest_pos + i, src.get(src_pos + i)));
    Ok(None)
}

#[no_mangle]
fn JAVA_LANG_SYSTEM_identity_hash_code(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_SYSTEM_map_library_name(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}
