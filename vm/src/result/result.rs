use crate::data_types::exception::Exception;
use std::{
    error,
    fmt::{self, Display, Formatter},
};

pub(crate) type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    JavaException,
    VMError(String),
    ParseError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self) }
}

impl error::Error for Error {}
