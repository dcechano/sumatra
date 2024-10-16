use std::collections::HashMap;

use anyhow::{bail, Result};

use crate::{
    data_types::{object::ObjRef, value::Value},
    native::{
        lib_java::{
            internal::{misc::java_unsafe, util::java_system_props_raw},
            lang::{
                java_class, java_double, java_float, java_object, java_string_utf16, java_system,
                java_throwable,
            },
            JAVA_LANG_CLASS, JAVA_LANG_DOUBLE, JAVA_LANG_FLOAT, JAVA_LANG_OBJECT,
            JAVA_LANG_STRING_UTF16, JAVA_LANG_SYSTEM, JAVA_LANG_THROWABLE,
            JDK_INTERNAL_MISC_UNSAFE, JDK_INTERNAL_SYSTEM_PROPS_RAW, REGISTER_NATIVES_SIG,
        },
        native_identifier::NativeIdentifier,
    },
    vm::VM,
};

const INITIAL_NATIVE_METHODS: [(&str, &str, NativeMethod); 12] = [
    (
        JAVA_LANG_OBJECT,
        java_object::GET_CLASS_SIG,
        java_object::jvm_get_class,
    ),
    (
        JAVA_LANG_OBJECT,
        java_object::HASH_CODE_SIG,
        java_object::jvm_hash_code,
    ),
    (
        JAVA_LANG_CLASS,
        REGISTER_NATIVES_SIG,
        java_class::jvm_register_natives,
    ),
    (
        JAVA_LANG_SYSTEM,
        REGISTER_NATIVES_SIG,
        java_system::jvm_register_natives,
    ),
    (
        JAVA_LANG_STRING_UTF16,
        java_string_utf16::IS_BIG_ENDIAN_SIG,
        java_string_utf16::jvm_is_big_endian,
    ),
    (
        JAVA_LANG_FLOAT,
        java_float::FLOAT_TO_RAW_INT_BITS_SIG,
        java_float::jvm_float_to_raw_int_bits,
    ),
    (
        JAVA_LANG_DOUBLE,
        java_double::DOUBLE_TO_RAW_LONG_BITS_SIG,
        java_double::jvm_double_to_raw_long_bits,
    ),
    (
        JAVA_LANG_DOUBLE,
        java_double::LONG_BITS_TO_DOUBLE_SIG,
        java_double::jvm_long_bits_to_double,
    ),
    (
        JAVA_LANG_THROWABLE,
        java_throwable::FILL_IN_STACK_TRACE,
        java_throwable::jvm_fill_in_stack_trace,
    ),
    (
        JDK_INTERNAL_SYSTEM_PROPS_RAW,
        java_system_props_raw::PLATFORM_PROPS_SIG,
        java_system_props_raw::jvm_platform_properties,
    ),
    (
        JDK_INTERNAL_SYSTEM_PROPS_RAW,
        java_system_props_raw::VM_PROPS_SIG,
        java_system_props_raw::jvm_vm_properties,
    ),
    (
        JDK_INTERNAL_MISC_UNSAFE,
        REGISTER_NATIVES_SIG,
        java_unsafe::jvm_register_natives,
    ),
];

pub type NativeMethod = fn(&mut VM, Option<ObjRef>, Vec<Value>) -> Result<Option<Value>>;

pub struct NativeRegistry {
    registry: HashMap<NativeIdentifier, NativeMethod>,
}

impl NativeRegistry {
    pub fn new() -> Self {
        let mut registery = Self {
            registry: HashMap::with_capacity(INITIAL_NATIVE_METHODS.len() * 32),
        };
        registery.register_native_registering_methods();
        registery
    }

    /// Registers the "native registerNatives()" java method.
    /// Calling the stored method will register the rest of the natives for that
    /// class.
    fn register_native_registering_methods(&mut self) {
        INITIAL_NATIVE_METHODS
            .iter()
            .for_each(|(class, sig, method)| {
                self.register(
                    NativeIdentifier::new(class.to_string(), sig.to_string()),
                    *method,
                )
            });
    }

    /// Retrieve a `NativeMethod` from the `NativeRegistry`.
    pub fn get(&self, native_id: &NativeIdentifier) -> Result<NativeMethod> {
        match self.registry.get(&native_id) {
            // Dereference so that we can still mutate the VM without the borrow checker considering
            // it borrowed while the NativeMethod is alive.
            Some(native) => Ok(*native),
            None => {
                bail!("Unable to find native method: {native_id:?}. Native may be unregistered.")
            }
        }
    }

    /// Register a `NativeMethod` to store it in the `NativeRegistry`.
    pub fn register(&mut self, native_identifier: NativeIdentifier, native_method: NativeMethod) {
        self.registry.insert(native_identifier, native_method);
    }
}
