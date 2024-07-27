use anyhow::Result;
use std::mem;

use crate::{
    data_types::{object::ObjRef, value::Value},
    native::{
        lib_java::JDK_INTERNAL_MISC_UNSAFE, native_identifier::NativeIdentifier,
        registry::NativeMethod,
    },
    vm::VM,
};

const NATIVES: [(&str, NativeMethod); 67] = [
    ("getInt(Ljava/lang/Object;J)I", jvm_get_int),
    ("putInt(Ljava/lang/Object;JI)V", jvm_put_int),
    ("getReference(Ljava/lang/Object;J)Ljava/lang/Object;", jvm_get_reference),
    ("putReference(Ljava/lang/Object;JLjava/lang/Object;)V", jvm_put_reference),
    ("getBoolean(Ljava/lang/Object;J)Z", jvm_get_boolean),
    ("putBoolean(Ljava/lang/Object;JZ)V", jvm_put_boolean),
    ("getByte(Ljava/lang/Object;J)B", jvm_get_byte),
    ("putByte(Ljava/lang/Object;JB)V", jvm_put_byte),
    ("getShort(Ljava/lang/Object;J)S", jvm_get_short),
    ("putShort(Ljava/lang/Object;JS)V", jvm_put_short),
    ("getChar(Ljava/lang/Object;J)C", jvm_get_char),
    ("putChar(Ljava/lang/Object;JC)V", jvm_put_char),
    ("getLong(Ljava/lang/Object;J)J", jvm_get_long),
    ("putLong(Ljava/lang/Object;JJ)V", jvm_put_long),
    ("getFloat(Ljava/lang/Object;J)J", jvm_get_float),
    ("putFloat(Ljava/lang/Object;JF)F", jvm_put_float),
    ("getDouble(Ljava/lang/Object;J)D", jvm_get_double),
    ("putDouble(Ljava/lang/Object;JD)V", jvm_put_double),
    ("getUncompressedObject(J)Ljava/lang/Object;", jvm_get_uncompressed_object),
    ("writeback0(J)V", jvm_write_back0),
    ("writebackPreSync0()V", jvm_write_back_pre_sync0),
    ("writebackPostSync0()V", jvm_write_back_post_sync0),
    ("defineClass0(Ljava/lang/String;[BIILjava/lang/ClassLoader;Ljava/security/ProtectionDomain;)Ljava/lang/Class", jvm_define_class0),
    ("allocateInstance(Ljava/lang/Class;)Ljava/lang/Object;", jvm_allocate_instance),
    ("throwException(Ljava/lang/Throwable;)", jvm_throw_exception),
    ("compareAndSetReference(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z", jvm_compare_and_set_reference),
    ("compareAndExchangeReference(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;", jvm_compare_and_exchange_reference),
    ("compareAndSetInt(Ljava/lang/Object;JII)Z", jvm_compare_and_set_int),
    ("compareAndExchangeInt(Ljava/lang/Object;JII)I", jvm_compare_and_exchange_int),
    ("compareAndSetLong(Ljava/lang/Object;JJJ)Z", jvm_compare_and_set_int),
    ("compareAndExchangeLong(Ljava/lang/Object;JJJ)J", jvm_compare_and_exchange_int),
    ("getReferenceVolatile(Ljava/lang/Object;J)Ljava/lang/Object;", jvm_get_reference_volatile),
    ("putReferenceVolatile(Ljava/lang/Object;JLjava/lang/Object;)V", jvm_put_reference_volatile),
    ("getIntVolatile(Ljava/lang/Object;J)I", jvm_get_int_volatile),
    ("putIntVolatile(Ljava/lang/Object;JI)V", jvm_put_int_volatile),
    ("getBooleanVolatile(Ljava/lang/Object;J)Z", jvm_get_boolean_volatile),
    ("putBooleanVolatile(Ljava/lang/Object;JZ)V", jvm_put_boolean_volatile),
    ("getByteVolatile(Ljava/lang/Object;J)B", jvm_get_byte_volatile),
    ("putByteVolatile(Ljava/lang/Object;JI)V", jvm_put_byte_volatile),
    ("getShortVolatile(Ljava/lang/Object;J)s", jvm_get_short_volatile),
    ("putShortVolatile(Ljava/lang/Object;Js)V", jvm_put_short_volatile),
    ("getCharVolatile(Ljava/lang/Object;J)C", jvm_get_char_volatile),
    ("putCharVolatile(Ljava/lang/Object;JC)V", jvm_put_char_volatile),
    ("getLongVolatile(Ljava/lang/Object;J)J", jvm_get_long_volatile),
    ("putLongVolatile(Ljava/lang/Object;JI)V", jvm_put_long_volatile),
    ("getFloatVolatile(Ljava/lang/Object;J)F", jvm_get_float_volatile),
    ("putFloatVolatile(Ljava/lang/Object;JF)V", jvm_put_float_volatile),
    ("getDoubleVolatile(Ljava/lang/Object;J)D", jvm_get_double_volatile),
    ("putDoubleVolatile(Ljava/lang/Object;JD)V", jvm_put_double_volatile),
    ("unpark(Ljava/lang/Object;)V", jvm_unpark),
    ("park(ZJ)V", jvm_park),
    ("fullFence()V", jvm_full_fence),
    ("allocateMemory0(J)J", jvm_allocate_memory0),
    ("reallocateMemory0(JJ)J", jvm_reallocate_memory0),
    ("freeMemory0(J)V", jvm_free_memory0),
    ("setMemory0(Ljava/lang/Object;JJB)V", jvm_free_memory0),
    ("copyMemory0(Ljava/lang/Object;JLjava/lang/Object;JJ)V", jvm_copy_memory0),
    ("copySwapMemory(Ljava/lang/Object;JLjava/lang/Object;JJJ)V", jvm_copy_swap_memory0),
    ("objectFieldOffset0(Ljava/lang/Field;)J", jvm_object_field_offset0),
    ("objectFieldOffset1(Ljava/lang/Class;Ljava/lang/String)J", jvm_object_field_offset1),
    ("staticFieldOffset0(Ljava/lang/Field)J", jvm_static_field_offset0),
    ("staticFieldBase1(Ljava/lang/Field)Ljava/lang/Object;", jvm_static_field_base0),
    ("shouldBeInitialized0(Ljava/lang/Class;)Z", jvm_should_be_initialized0),
    ("ensureClassInitialized0(Ljava/lang/Class;)V", jvm_ensure_class_initialized0),
    ("arrayBaseOffset0(Ljava/lang/Class;)I", jvm_array_base_offset0),
    ("arrayIndexScale0(Ljava/lang/Class;)I", jvm_array_index_scale0),
    ("getLoadAverage(DI)V", jvm_get_load_average0)
];

pub fn jvm_register_natives(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    NATIVES.iter().for_each(|(name, method)| {
        vm.native_registry.register(
            NativeIdentifier::new(JDK_INTERNAL_MISC_UNSAFE.to_string(), name.to_string()),
            *method,
        );
    });

    Ok(None)
}

fn jvm_get_int(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_int(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_reference(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_reference(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_boolean(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_boolean(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_byte(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_byte(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_short(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_short(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_char(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_char(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_long(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_long(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_float(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_float(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_double(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_double(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_uncompressed_object(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_write_back0(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_write_back_pre_sync0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_write_back_post_sync0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_define_class0(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_allocate_instance(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_throw_exception(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_compare_and_set_reference(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_compare_and_exchange_reference(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_compare_and_set_int(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_compare_and_exchange_int(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_compare_and_set_long(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_compare_and_exchange_long(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_reference_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_reference_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_int_volatile(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_int_volatile(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_boolean_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_boolean_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_byte_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_byte_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_short_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_short_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_char_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_char_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_long_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_long_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_float_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_float_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_double_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_put_double_volatile(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_unpark(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> { todo!() }

fn jvm_park(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> { todo!() }

fn jvm_full_fence(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_allocate_memory0(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_reallocate_memory0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_free_memory0(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_set_memory0(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_copy_memory0(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

fn jvm_copy_swap_memory0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_object_field_offset0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_object_field_offset1(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_static_field_offset0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_static_field_base0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_should_be_initialized0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_ensure_class_initialized0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_array_base_offset0(_: &mut VM, _: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    Ok(Some(Value::Int(size_of::<*mut Value>() as i32)))
}

fn jvm_array_index_scale0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

fn jvm_get_load_average0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}
