use sumatra_vm::{
    data_types::{object::ObjRef, value::Value},
    result::Result,
    vm::VM,
};

#[no_mangle]
pub(crate) fn JAVA_IO_FILEDESCRIPTOR_sync0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub(crate) fn JAVA_IO_FILEDESCRIPTOR_init_i_ds(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub(crate) fn JAVA_IO_FILEDESCRIPTOR_get_handle(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub(crate) fn JAVA_IO_FILEDESCRIPTOR_get_append(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}

#[no_mangle]
pub(crate) fn JAVA_IO_FILEDESCRIPTOR_close0(
    _: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    todo!()
}
