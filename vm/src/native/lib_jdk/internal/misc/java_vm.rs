use crate::{
    data_types::{object::ObjRef, value::Value},
    native::{
        lib_jdk::JDK_INTERNAL_MISC_VM, native_identifier::NativeIdentifier, registry::NativeMethod,
    },
    vm::VM,
};
use anyhow::Result;

const NATIVES: [(&str, NativeMethod); 7] = [
    (
        "latestUserDefinedLoader0()Ljava/lang/ClassLoader;",
        jvm_latest_user_defined_loader0,
    ),
    ("getuid()J", jvm_getuid),
    ("geteuid()J", jvm_geteuid),
    ("getgid()J", jvm_getgid),
    ("getegid()J", jvm_getegid),
    ("getNanoTimeAdjustment(J)J", jvm_get_nano_time_adjustment),
    (
        "getRuntimeArguments()[Ljava/lang/String;",
        jvm_get_runtime_arguments,
    ),
];

pub const INITIALIZE_SIG: &str = "initialize()V";

pub fn jvm_initialize(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    NATIVES.iter().for_each(|(name, method)| {
        vm.native_registry.register(
            NativeIdentifier::new(JDK_INTERNAL_MISC_VM.to_string(), name.to_string()),
            *method,
        );
    });
    Ok(None)
}

pub fn jvm_latest_user_defined_loader0(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

pub fn jvm_getuid(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

pub fn jvm_geteuid(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

pub fn jvm_getgid(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

pub fn jvm_getegid(vm: &mut VM, this: Option<ObjRef>, _: Vec<Value>) -> Result<Option<Value>> {
    todo!()
}

pub fn jvm_get_nano_time_adjustment(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

pub fn jvm_get_runtime_arguments(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}
