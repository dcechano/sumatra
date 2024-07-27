use anyhow::Result;

use crate::{
    data_types::{
        object::ObjRef,
        value::{RefType, Value},
    },
    native::{
        lib_java::JAVA_LANG_CLASS, native_identifier::NativeIdentifier, registry::NativeMethod,
    },
    vm::VM,
};

const NATIVES: [(&str, NativeMethod); 4] = [
    (
        "forName0(Ljava/lang/String;ZLjava/lang/ClassLoader;Ljava/lang/Class;)Ljava/lang/Class;",
        jvm_for_name0,
    ),
    ("isInstance(Ljava/lang/Object;)Z", jvm_is_instance),
    (
        "desiredAssertionStatus0(Ljava/lang/Class;)Z",
        jvm_desired_assertion_status0,
    ),
    (
        "getPrimitiveClass(Ljava/lang/String;)Ljava/lang/Class;",
        jvm_get_primitive_class,
    ),
];

pub fn jvm_register_natives(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    NATIVES.iter().for_each(|(name, method)| {
        vm.native_registry.register(
            NativeIdentifier::new(JAVA_LANG_CLASS.to_string(), name.to_string()),
            *method,
        );
    });

    Ok(None)
}

/// Runs `private static native Class<?> forName0(String name, boolean
/// initialize,                                             ClassLoader loader,
///                                            Class<?> caller)`
/// in Class.java
fn jvm_for_name0(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

/// Runs ` public native boolean isInstance(Object obj);` in Class.java
fn jvm_is_instance(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_is_assignable_from(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_is_interface(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_is_array(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_is_primitive(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_init_class_name(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_super_class(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_interfaces(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_modifiers(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_signers(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_set_signers(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_enclosing_method(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_declaring_class(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_simple_binary_name(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_protection_domain(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_primitive_class(
    vm: &mut VM,
    this: Option<ObjRef>,
    args: Vec<Value>,
) -> Result<Option<Value>> {
    let Value::Ref(RefType::Object(string_obj)) = &args[0] else {
        panic!("Arg was not a RefType::Object as expected in jvm_get_primitive_class");
    };
    let Value::Ref(RefType::Array(bytes)) = string_obj.get_field("value").unwrap() else {
        panic!("bytes was not a RefType::Array as expected in jvm_get_primitive_class");
    };
    let bytes = bytes.get_all();

    let primitive_name = bytes
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            let Value::Byte(byte) = byte else {
                panic!("Expected a Value::Byte in jvm_get_primitive_class. Got {byte:?}");
            };
            char::from(*byte as u8)
        })
        .collect::<String>();

    //TODO fix this. int.class does NOT return a java/lang.Integer.class!
    let class_name = match primitive_name.as_bytes() {
        b"boolean" => "java/lang/Boolean",
        b"byte" => "java/lang/Byte",
        b"char" => "java/lang/Char",
        b"double" => "java/lang/Double",
        b"float" => "java/lang/Float",
        b"int" => "java/lang/Integer",
        b"long" => "java/lang/Long",
        b"short" => "java/lang/Short",
        b"void" => "java/lang/Void",
        primitive => panic!("Invalid primitive type: {primitive:?} in jvm_get_primitive_class."),
    };
    let string_obj = vm.heap().get_class_obj(class_name);
    Ok(Some(Value::new_object(string_obj)))
}

fn jvm_get_generic_signature(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_raw_annotations(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_raw_type_annotions(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_constant_pool(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_declared_fields(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_declared_methods(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_declared_constructors(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_record_components(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_is_record(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_desired_assertion_status0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    // Return false as the desired assertion status.
    // Tbh, I am not completely sure if I need this to be true, false, or it does
    // not matter.
    Ok(Some(Value::Int(0)))
}

fn jvm_get_nest_host(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_nested_members(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_is_hidden(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_permitted_subclasses(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_file_version(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_class_access_flags_raw(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}
