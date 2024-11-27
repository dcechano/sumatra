use anyhow::Result;

use crate::{
    data_types::{
        object::ObjRef,
        value::{RefType, Value},
    },
    lli::class_loader,
    native::{
        native_identifier::NativeIdentifier,
        registry::NativeMethod,
    },
    vm::VM,
};

pub(crate) const AVAILABLE_PROCESSORS_SIG: &str = "availableProcessors()I";
pub(crate) const FREE_MEMORY_SIG: &str = "freeMemory()J";
pub(crate) const TOTAL_MEMORY_SIG: &str = "totalMemory()J";
pub(crate) const MAX_MEMORY_SIG: &str = "maxMemory()J";
pub(crate) const GC_SIG: &str = "gc()V";

pub fn jvm_available_processors(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

pub fn jvm_free_memory(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

pub fn jvm_total_memory(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    //TODO reimplement proper once there is
    //tracking for how much memory has been used.
    Ok(Some(Value::Long(i64::MAX)))
}

pub fn jvm_max_memory(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    //TODO reimplement proper once there is
    //tracking for how much memory is allowed to be used.
    Ok(Some(Value::Long(i64::MAX)))
}

pub fn jvm_gc(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> { todo!() }
