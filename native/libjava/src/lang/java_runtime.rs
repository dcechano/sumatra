use sumatra_vm::{
    data_types::{object::ObjRef, value::Value},
    lli::class_loader,
    result::Result,
    vm::VM,
};

pub(crate) const AVAILABLE_PROCESSORS_SIG: &str = "availableProcessors()I";
pub(crate) const FREE_MEMORY_SIG: &str = "freeMemory()J";
pub(crate) const TOTAL_MEMORY_SIG: &str = "totalMemory()J";
pub(crate) const MAX_MEMORY_SIG: &str = "maxMemory()J";
pub(crate) const GC_SIG: &str = "gc()V";

#[no_mangle]
pub fn JAVA_LANG_RUNTIME_available_processors(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    //TODO: I am not sure I like importing a crate just for this one call.
    let num_procs = num_cpus::get_physical();
    Ok(Some(Value::Int(num_procs as i32)))
}

#[no_mangle]
pub fn JAVA_LANG_RUNTIME_free_memory(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub fn JAVA_LANG_RUNTIME_total_memory(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    //TODO reimplement proper once there is
    //tracking for how much memory has been used.
    Ok(Some(Value::Long(i64::MAX)))
}

#[no_mangle]
pub fn JAVA_LANG_RUNTIME_max_memory(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    //TODO reimplement proper once there is
    //tracking for how much memory is allowed to be used.
    Ok(Some(Value::Long(i64::MAX)))
}

#[no_mangle]
pub fn JAVA_LANG_RUNTIME_gc(
    vm: &mut VM,
    this: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}
