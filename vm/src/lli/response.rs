use crate::class::Class;

pub(crate) enum Response {
    InitReq(&'static Class, usize),
    /// InitReqArray(ArrayClass, index of array_class, Component class with its
    /// index (if applicable))
    InitReqArray(&'static Class, usize, Option<(&'static Class, usize)>),
    NotFound,
    Ready(usize),
}
