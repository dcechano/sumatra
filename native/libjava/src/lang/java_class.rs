use sumatra_vm::{
    data_types::{
        object::ObjRef,
        value::{RefType, Value},
    },
    native::NativeMethod,
    result::Result,
    vm::VM,
};

const NATIVES: [(&str, NativeMethod); 6] = [
    (
        "forName0(Ljava/lang/String;ZLjava/lang/ClassLoader;Ljava/lang/Class;)Ljava/lang/Class;",
        JAVA_LANG_CLASS_for_name0,
    ),
    (
        "isInstance(Ljava/lang/Object;)Z",
        JAVA_LANG_CLASS_is_instance,
    ),
    (
        "desiredAssertionStatus0(Ljava/lang/Class;)Z",
        JAVA_LANG_CLASS_desired_assertion_status0,
    ),
    (
        "getPrimitiveClass(Ljava/lang/String;)Ljava/lang/Class;",
        JAVA_LANG_CLASS_get_primitive_class,
    ),
    (
        "initClassName()Ljava/lang/String;",
        JAVA_LANG_CLASS_init_class_name,
    ),
    ("isPrimitive()Z", JAVA_LANG_CLASS_is_primitive),
];

#[no_mangle]
fn JAVA_LANG_CLASS_register_natives(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    Ok(None)
}

/// Runs `private static native Class<?> forName0(String name, boolean
/// initialize,                                             ClassLoader loader,
///                                            Class<?> caller)`
/// in Class.java
#[no_mangle]
fn JAVA_LANG_CLASS_for_name0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

/// Runs ` public native boolean isInstance(Object obj);` in Class.java
#[no_mangle]
fn JAVA_LANG_CLASS_is_instance(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_is_assignable_from(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_is_interface(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_is_array(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_is_primitive(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    const PRIMS: [&str; 8] = [
        "boolean", "byte", "char", "double", "float", "int", "long", "short",
    ];

    let name = vm.heap().class_name(this.unwrap());
    if PRIMS.contains(&name.as_str()) {
        return Ok(Some(Value::Int(1)));
    }
    Ok(Some(Value::Int(0)))
}

#[no_mangle]
fn JAVA_LANG_CLASS_init_class_name(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    let mut this = this.unwrap();
    let class_name = vm.heap().class_name(this);
    let java_string = vm.create_java_string(&class_name, false);

    let field = Value::new_object(java_string);
    this.set_field("name", field.clone()).unwrap();
    Ok(Some(field))
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_super_class(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_interfaces(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_modifiers(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_signers(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_set_signers(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_enclosing_method(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_declaring_class(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_simple_binary_name(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_protection_domain(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_primitive_class(
    vm: &mut VM,
    this: Option<ObjRef>,
    args: Vec<Value>,
) -> Result<Option<Value>> {
    let Value::Ref(RefType::Object(string_obj)) = &args[0] else {
        panic!("Arg was not a RefType::Object as expected in JAVA_LANG_CLASS_get_primitive_class");
    };
    let Value::Ref(RefType::Array(bytes)) = string_obj.get_field("value").unwrap() else {
        panic!("bytes was not a RefType::Array as expected in JAVA_LANG_CLASS_get_primitive_class");
    };
    let bytes = bytes.get_all();
    let primitive_name = bytes
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            let Value::Byte(byte) = byte else {
                panic!(
                    "Expected a Value::Byte in JAVA_LANG_CLASS_get_primitive_class. Got {byte:?}"
                );
            };
            char::from(*byte as u8)
        })
        .collect::<String>();

    let string_obj = vm.heap().get_class_obj(&primitive_name);
    Ok(Some(Value::new_object(string_obj)))
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_generic_signature(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_raw_annotations(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_raw_type_annotions(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_constant_pool(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_declared_fields(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_declared_methods(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_declared_constructors(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_record_components(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_is_record(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_desired_assertion_status0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    // Return false as the desired assertion status.
    // Tbh, I am not completely sure if I need this to be true, false, or it does
    // not matter.
    Ok(Some(Value::Int(0)))
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_nest_host(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_nested_members(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_is_hidden(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_permitted_subclasses(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_file_version(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JAVA_LANG_CLASS_get_class_access_flags_raw(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}
