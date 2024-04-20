use crate::alloc::{alloc_type::NonStatic, oop::HeapAlloc};

#[derive(Default, Debug, Clone)]
pub enum Value {
    #[default]
    Null,
    Double(f64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Long(i64),
    Float(f32),
    StringConst(String),
    Ref(*mut HeapAlloc<NonStatic>),
}
