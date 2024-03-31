use crate::oop::HeapAlloc;

#[derive(Default, Debug)]
#[repr(C)]
pub enum Value<'data> {
    #[default]
    Null,
    Double(f64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Long(i64),
    RefType(HeapAlloc<'data>),
}
