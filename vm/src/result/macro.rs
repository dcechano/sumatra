#[macro_export]
macro_rules! vm_error  {
    () => {
       return ::core::result::Result::Err($crate::result::Error::VMError(""));
    };
    ($fmt:expr) => {
       return ::core::result::Result::Err($crate::result::Error::VMError(format!($fmt)));
    };
    ($fmt:expr, $($arg:tt)*) => {
       return Err($crate::result::Error::VMError(format!($fmt, $($arg)*)));
    };
}

#[macro_export]
macro_rules! parse_error  {
    () => {
       return ::core::result::Result::Err($crate::result::Error::ParseError(""));
    };
    ($fmt:expr) => {
       return ::core::result::Result::Err($crate::result::Error::ParseError(format!($fmt)));
    };
    ($fmt:expr, $($arg:tt)*) => {
       return Err($crate::result::Error::ParseError(format!($fmt, $($arg)*)));
    };
}

#[macro_export]
macro_rules! jexcept  {
    () => {
       return ::core::result::Result::Err($crate::result::Error::JavaException(""));
    };
    ($fmt:expr) => {
       return ::core::result::Result::Err($crate::result::Error::JavaException(format!($fmt)));
    };
    ($fmt:expr, $($arg:tt)*) => {
       return Err($crate::result::Error::JavaException(format!($fmt, $($arg)*)));
    };
}
