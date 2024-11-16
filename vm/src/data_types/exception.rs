use super::object::ObjRef;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Exception {
    NullPointer(ObjRef),
    OutOfBounds(ObjRef),
    Runtime(ObjRef),
}

impl Exception {
    pub(crate) fn inner(&self) -> ObjRef {
        let obj = match self {
            Exception::NullPointer(obj) => obj,
            Exception::OutOfBounds(obj) => obj,
            Exception::Runtime(obj) => obj,
        };
        *obj
    }
}
