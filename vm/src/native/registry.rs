use std::collections::HashMap;

use anyhow::{bail, Result};

use crate::{
    data_types::{reference_types::ObjRef, value::Value},
    native::{
        lib_java::{java_class, java_object, JAVA_LANG_CLASS, JAVA_LANG_OBJECT},
        native_identifier::NativeIdentifier,
    },
    vm::VM,
};

const INITIAL_NATIVE_METHODS: [(&str, &str, NativeMethod); 2] = [
    (
        JAVA_LANG_OBJECT,
        java_object::GET_CLASS_SIG,
        java_object::jvm_get_class,
    ),
    (
        JAVA_LANG_CLASS,
        java_class::REGISTER_NATIVES_SIG,
        java_class::jvm_register_natives,
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
