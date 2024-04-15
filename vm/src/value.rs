use crate::alloc::{alloc_type::NonStatic, oop::HeapAlloc};

#[derive(Default, Debug)]
pub enum Value {
    #[default]
    Null,
    Double(f64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Long(i64),
    RefType(HeapAlloc<NonStatic>),
}
