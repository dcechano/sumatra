use anyhow::Result;
use sumatra_vm::{
    data_types::{object::ObjRef, value::Value},
    native::NativeMethod,
    vm::VM,
};

const NATIVES: [(&str, NativeMethod); 7] = [
    (
        "latestUserDefinedLoader0()Ljava/lang/ClassLoader;",
        JDK_INTERNAL_MISC_VM_latest_user_defined_loader0,
    ),
    ("getuid()J", JDK_INTERNAL_MISC_VM_getuid),
    ("geteuid()J", JDK_INTERNAL_MISC_VM_geteuid),
    ("getgid()J", JDK_INTERNAL_MISC_VM_getgid),
    ("getegid()J", JDK_INTERNAL_MISC_VM_getegid),
    (
        "getNanoTimeAdjustment(J)J",
        JDK_INTERNAL_MISC_VM_get_nano_time_adjustment,
    ),
    (
        "getRuntimeArguments()[Ljava/lang/String;",
        JDK_INTERNAL_MISC_VM_get_runtime_arguments,
    ),
];

pub const INITIALIZE_SIG: &str = "initialize()V";

#[no_mangle]
pub fn JDK_INTERNAL_MISC_VM_initialize(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    println!("java_vm.rs is do a noop for initialize.");
    Ok(None)
}

#[no_mangle]
pub fn JDK_INTERNAL_MISC_VM_latest_user_defined_loader0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub fn JDK_INTERNAL_MISC_VM_getuid(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub fn JDK_INTERNAL_MISC_VM_geteuid(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub fn JDK_INTERNAL_MISC_VM_getgid(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub fn JDK_INTERNAL_MISC_VM_getegid(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub fn JDK_INTERNAL_MISC_VM_get_nano_time_adjustment(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub fn JDK_INTERNAL_MISC_VM_get_runtime_arguments(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}
