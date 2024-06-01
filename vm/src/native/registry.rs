use crate::{
    native::native_identifier::NativeIdentifier, reference_types::ObjRef, value::Value, vm::VM,
};
use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;

type NativeMethod = fn(&mut VM, Option<ObjRef>, Vec<Value>) -> Result<Option<Value>>;

pub struct Registry {
    registry: HashMap<NativeIdentifier, NativeMethod>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            registry: HashMap::with_capacity(128),
        }
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
