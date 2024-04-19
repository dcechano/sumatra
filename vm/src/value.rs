use crate::alloc::{alloc_type::NonStatic, oop::HeapAlloc};

#[derive(Default, Debug, Clone, Copy)]
pub enum Value {
    #[default]
    Null,
    Double(f64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Long(i64),
    Ref(*mut HeapAlloc<NonStatic>),
}
