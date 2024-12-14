use anyhow::Result;

use sumatra_vm::{
    data_types::{object::ObjRef, value::Value},
    native::NativeMethod,
    vm::VM,
};

const NATIVES: [(&str, NativeMethod); 67] = [
    ("getInt(Ljava/lang/Object;J)I", JDK_INTERNAL_MISC_UNSAFE_get_int),
    ("putInt(Ljava/lang/Object;JI)V", JDK_INTERNAL_MISC_UNSAFE_put_int),
    ("getReference(Ljava/lang/Object;J)Ljava/lang/Object;", JDK_INTERNAL_MISC_UNSAFE_get_reference),
    ("putReference(Ljava/lang/Object;JLjava/lang/Object;)V", JDK_INTERNAL_MISC_UNSAFE_put_reference),
    ("getBoolean(Ljava/lang/Object;J)Z", JDK_INTERNAL_MISC_UNSAFE_get_boolean),
    ("putBoolean(Ljava/lang/Object;JZ)V", JDK_INTERNAL_MISC_UNSAFE_put_boolean),
    ("getByte(Ljava/lang/Object;J)B", JDK_INTERNAL_MISC_UNSAFE_get_byte),
    ("putByte(Ljava/lang/Object;JB)V", JDK_INTERNAL_MISC_UNSAFE_put_byte),
    ("getShort(Ljava/lang/Object;J)S", JDK_INTERNAL_MISC_UNSAFE_get_short),
    ("putShort(Ljava/lang/Object;JS)V", JDK_INTERNAL_MISC_UNSAFE_put_short),
    ("getChar(Ljava/lang/Object;J)C", JDK_INTERNAL_MISC_UNSAFE_get_char),
    ("putChar(Ljava/lang/Object;JC)V", JDK_INTERNAL_MISC_UNSAFE_put_char),
    ("getLong(Ljava/lang/Object;J)J", JDK_INTERNAL_MISC_UNSAFE_get_long),
    ("putLong(Ljava/lang/Object;JJ)V", JDK_INTERNAL_MISC_UNSAFE_put_long),
    ("getFloat(Ljava/lang/Object;J)J", JDK_INTERNAL_MISC_UNSAFE_get_float),
    ("putFloat(Ljava/lang/Object;JF)F", JDK_INTERNAL_MISC_UNSAFE_put_float),
    ("getDouble(Ljava/lang/Object;J)D", JDK_INTERNAL_MISC_UNSAFE_get_double),
    ("putDouble(Ljava/lang/Object;JD)V", JDK_INTERNAL_MISC_UNSAFE_put_double),
    ("getUncompressedObject(J)Ljava/lang/Object;", JDK_INTERNAL_MISC_UNSAFE_get_uncompressed_object),
    ("writeback0(J)V", JDK_INTERNAL_MISC_UNSAFE_write_back0),
    ("writebackPreSync0()V", JDK_INTERNAL_MISC_UNSAFE_write_back_pre_sync0),
    ("writebackPostSync0()V", JDK_INTERNAL_MISC_UNSAFE_write_back_post_sync0),
    ("defineClass0(Ljava/lang/String;[BIILjava/lang/ClassLoader;Ljava/security/ProtectionDomain;)Ljava/lang/Class", JDK_INTERNAL_MISC_UNSAFE_define_class0),
    ("allocateInstance(Ljava/lang/Class;)Ljava/lang/Object;", JDK_INTERNAL_MISC_UNSAFE_allocate_instance),
    ("throwException(Ljava/lang/Throwable;)", JDK_INTERNAL_MISC_UNSAFE_throw_exception),
    ("compareAndSetReference(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z", JDK_INTERNAL_MISC_UNSAFE_compare_and_set_reference),
    ("compareAndExchangeReference(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;", JDK_INTERNAL_MISC_UNSAFE_compare_and_exchange_reference),
    ("compareAndSetInt(Ljava/lang/Object;JII)Z", JDK_INTERNAL_MISC_UNSAFE_compare_and_set_int),
    ("compareAndExchangeInt(Ljava/lang/Object;JII)I", JDK_INTERNAL_MISC_UNSAFE_compare_and_exchange_int),
    ("compareAndSetLong(Ljava/lang/Object;JJJ)Z", JDK_INTERNAL_MISC_UNSAFE_compare_and_set_int),
    ("compareAndExchangeLong(Ljava/lang/Object;JJJ)J", JDK_INTERNAL_MISC_UNSAFE_compare_and_exchange_int),
    ("getReferenceVolatile(Ljava/lang/Object;J)Ljava/lang/Object;", JDK_INTERNAL_MISC_UNSAFE_get_reference_volatile),
    ("putReferenceVolatile(Ljava/lang/Object;JLjava/lang/Object;)V", JDK_INTERNAL_MISC_UNSAFE_put_reference_volatile),
    ("getIntVolatile(Ljava/lang/Object;J)I", JDK_INTERNAL_MISC_UNSAFE_get_int_volatile),
    ("putIntVolatile(Ljava/lang/Object;JI)V", JDK_INTERNAL_MISC_UNSAFE_put_int_volatile),
    ("getBooleanVolatile(Ljava/lang/Object;J)Z", JDK_INTERNAL_MISC_UNSAFE_get_boolean_volatile),
    ("putBooleanVolatile(Ljava/lang/Object;JZ)V", JDK_INTERNAL_MISC_UNSAFE_put_boolean_volatile),
    ("getByteVolatile(Ljava/lang/Object;J)B", JDK_INTERNAL_MISC_UNSAFE_get_byte_volatile),
    ("putByteVolatile(Ljava/lang/Object;JI)V", JDK_INTERNAL_MISC_UNSAFE_put_byte_volatile),
    ("getShortVolatile(Ljava/lang/Object;J)s", JDK_INTERNAL_MISC_UNSAFE_get_short_volatile),
    ("putShortVolatile(Ljava/lang/Object;Js)V", JDK_INTERNAL_MISC_UNSAFE_put_short_volatile),
    ("getCharVolatile(Ljava/lang/Object;J)C", JDK_INTERNAL_MISC_UNSAFE_get_char_volatile),
    ("putCharVolatile(Ljava/lang/Object;JC)V", JDK_INTERNAL_MISC_UNSAFE_put_char_volatile),
    ("getLongVolatile(Ljava/lang/Object;J)J", JDK_INTERNAL_MISC_UNSAFE_get_long_volatile),
    ("putLongVolatile(Ljava/lang/Object;JI)V", JDK_INTERNAL_MISC_UNSAFE_put_long_volatile),
    ("getFloatVolatile(Ljava/lang/Object;J)F", JDK_INTERNAL_MISC_UNSAFE_get_float_volatile),
    ("putFloatVolatile(Ljava/lang/Object;JF)V", JDK_INTERNAL_MISC_UNSAFE_put_float_volatile),
    ("getDoubleVolatile(Ljava/lang/Object;J)D", JDK_INTERNAL_MISC_UNSAFE_get_double_volatile),
    ("putDoubleVolatile(Ljava/lang/Object;JD)V", JDK_INTERNAL_MISC_UNSAFE_put_double_volatile),
    ("unpark(Ljava/lang/Object;)V", JDK_INTERNAL_MISC_UNSAFE_unpark),
    ("park(ZJ)V", JDK_INTERNAL_MISC_UNSAFE_park),
    ("fullFence()V", JDK_INTERNAL_MISC_UNSAFE_full_fence),
    ("allocateMemory0(J)J", JDK_INTERNAL_MISC_UNSAFE_allocate_memory0),
    ("reallocateMemory0(JJ)J", JDK_INTERNAL_MISC_UNSAFE_reallocate_memory0),
    ("freeMemory0(J)V", JDK_INTERNAL_MISC_UNSAFE_free_memory0),
    ("setMemory0(Ljava/lang/Object;JJB)V", JDK_INTERNAL_MISC_UNSAFE_free_memory0),
    ("copyMemory0(Ljava/lang/Object;JLjava/lang/Object;JJ)V", JDK_INTERNAL_MISC_UNSAFE_copy_memory0),
    ("copySwapMemory(Ljava/lang/Object;JLjava/lang/Object;JJJ)V", JDK_INTERNAL_MISC_UNSAFE_copy_swap_memory0),
    ("objectFieldOffset0(Ljava/lang/Field;)J", JDK_INTERNAL_MISC_UNSAFE_object_field_offset0),
    ("objectFieldOffset1(Ljava/lang/Class;Ljava/lang/String;)J", JDK_INTERNAL_MISC_UNSAFE_object_field_offset1),
    ("staticFieldOffset0(Ljava/lang/Field)J", JDK_INTERNAL_MISC_UNSAFE_static_field_offset0),
    ("staticFieldBase1(Ljava/lang/Field)Ljava/lang/Object;", JDK_INTERNAL_MISC_UNSAFE_static_field_base0),
    ("shouldBeInitialized0(Ljava/lang/Class;)Z", JDK_INTERNAL_MISC_UNSAFE_should_be_initialized0),
    ("ensureClassInitialized0(Ljava/lang/Class;)V", JDK_INTERNAL_MISC_UNSAFE_ensure_class_initialized0),
    ("arrayBaseOffset0(Ljava/lang/Class;)I", JDK_INTERNAL_MISC_UNSAFE_array_base_offset0),
    ("arrayIndexScale0(Ljava/lang/Class;)I", JDK_INTERNAL_MISC_UNSAFE_array_index_scale0),
    ("getLoadAverage(DI)V", JDK_INTERNAL_MISC_UNSAFE_get_load_average0)
];

#[no_mangle]
pub fn JDK_INTERNAL_MISC_UNSAFE_register_natives(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    println!("java_unsafe.rs is doint a noop for register_natives.");

    Ok(None)
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_int(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_int(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_reference(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_reference(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_boolean(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_boolean(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_byte(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_byte(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_short(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_short(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_char(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_char(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_long(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_long(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_float(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_float(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_double(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_double(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_uncompressed_object(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_write_back0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_write_back_pre_sync0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_write_back_post_sync0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_define_class0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_allocate_instance(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_throw_exception(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_compare_and_set_reference(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_compare_and_exchange_reference(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_compare_and_set_int(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_compare_and_exchange_int(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_compare_and_set_long(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_compare_and_exchange_long(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_reference_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_reference_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_int_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_int_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_boolean_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_boolean_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_byte_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_byte_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_short_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_short_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_char_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_char_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_long_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_long_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_float_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_float_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_double_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_put_double_volatile(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_unpark(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_park(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_full_fence(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    //TODO may need to be fixed when we implement multithreading
    Ok(None)
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_allocate_memory0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_reallocate_memory0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_free_memory0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_set_memory0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_copy_memory0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_copy_swap_memory0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_object_field_offset0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_object_field_offset1(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_static_field_offset0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_static_field_base0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_should_be_initialized0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_ensure_class_initialized0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_array_base_offset0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    Ok(Some(Value::Int(size_of::<*mut Value>() as i32)))
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_array_index_scale0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    //TODO not sure if this is correct. Docs say something about a nuance with
    // 'narrow' types. I'm not sure what that means.
    // https://github.com/openjdk/jdk/blob/master/src/java.base/share/classes/jdk/internal/misc/Unsafe.java#L1230
    Ok(Some(Value::Int(size_of::<*mut Value>() as i32)))
}

#[no_mangle]
fn JDK_INTERNAL_MISC_UNSAFE_get_load_average0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}
