use crate::class::Class;

pub(crate) enum Response {
    InitReq(&'static Class, usize),
    NotFound,
    Ready(usize),
}
