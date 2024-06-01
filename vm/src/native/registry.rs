use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::{
    native::{
        lib_java::{java_class, JAVA_LANG_CLASS},
        native_identifier::NativeIdentifier,
    },
    reference_types::ObjRef,
    value::Value,
    vm::VM,
};

const REGISTER_NATIVES_METHOD_SIG: &str = "registerNatives()V";

const NATIVE_REGISTERING_METHODS: [(&str, NativeMethod); 1] =
    [(JAVA_LANG_CLASS, java_class::jvm_register_natives)];

pub type NativeMethod = fn(&mut VM, Option<ObjRef>, Vec<Value>) -> Result<Option<Value>>;

pub struct Registry {
    registry: HashMap<NativeIdentifier, NativeMethod>,
}

impl Registry {
    pub fn new() -> Self {
        let mut registery = Self {
            registry: HashMap::with_capacity(NATIVE_REGISTERING_METHODS.len() * 32),
        };
        registery.register_native_registering_methods();
        registery
    }

    fn register_native_registering_methods(&mut self) {
        NATIVE_REGISTERING_METHODS
            .iter()
            .for_each(|(class, method)| {
                self.register(
                    NativeIdentifier::new(
                        class.to_string(),
                        REGISTER_NATIVES_METHOD_SIG.to_string(),
                    ),
                    *method,
                )
            });
    }

    pub fn get(&self, native_identifier: &NativeIdentifier) -> Result<&NativeMethod> {
        self.registry.get(&native_identifier).ok_or(anyhow!(
            "Unable to find native method: {native_identifier:?}. Native may be unregistered."
        ))
    }

    pub fn register(&mut self, native_identifier: NativeIdentifier, native_method: NativeMethod) {
        self.registry.insert(native_identifier, native_method);
    }
}
