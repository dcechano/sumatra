use crate::class::Class;

pub(crate) enum Response {
    InitReq(&'static Class, usize),
    NonFound,
    Ready(usize),
}
