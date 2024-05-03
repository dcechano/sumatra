use crate::alloc::{alloc_type::NonStatic, oop::HeapAlloc};
use std::cmp::Ordering;

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
    ReturnAddress(usize),
    Ref(*mut HeapAlloc<NonStatic>),
}

impl Value {
    pub fn default_vec(cap: usize) -> Vec<Value> { vec![Value::Null; cap] }

    pub fn populate_locals(num_locals: usize, params: &mut Vec<Value>) {
        if params.len() > num_locals {
            panic!("The number of locals cannot be the greater than the number of params.");
        }

        let num_dummies = num_locals - params.len();
        params.extend(vec![Value::Null; num_dummies]);
    }

    fn is_same_variant(&self, other: &Value) -> bool {
        match self {
            Value::Null => matches!(other, Value::Null),
            Value::Double(_) => matches!(other, Value::Double(_)),
            Value::Int(_) => matches!(other, Value::Int(_)),
            Value::Short(_) => matches!(other, Value::Short(_)),
            Value::Byte(_) => matches!(other, Value::Byte(_)),
            Value::Long(_) => matches!(other, Value::Long(_)),
            Value::Float(_) => matches!(other, Value::Float(_)),
            Value::StringConst(_) => matches!(other, Value::StringConst(_)),
            Value::ReturnAddress(_) => matches!(other, Value::ReturnAddress(_)),
            Value::Ref(_) => matches!(other, Value::Ref(_)),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if !self.is_same_variant(other) {
            return false;
        }

        match self {
            Value::Null => true,
            Value::Double(double) => {
                if let Value::Double(other) = other {
                    double.partial_cmp(other) == Some(Ordering::Equal)
                } else {
                    unreachable!()
                }
            }
            Value::Int(int) => {
                if let Value::Int(other) = other {
                    *int == *other
                } else {
                    unreachable!()
                }
            }
            Value::Short(short) => {
                if let Value::Short(other) = other {
                    *short == *other
                } else {
                    unreachable!()
                }
            }
            Value::Byte(byte) => {
                if let Value::Byte(other) = other {
                    *byte == *other
                } else {
                    unreachable!()
                }
            }
            Value::Long(long) => {
                if let Value::Long(other) = other {
                    *long == *other
                } else {
                    unreachable!()
                }
            }
            Value::Float(float) => {
                if let Value::Float(other) = other {
                    float.partial_cmp(other) == Some(Ordering::Equal)
                } else {
                    unreachable!()
                }
            }
            Value::StringConst(string) => {
                //TODO this may need to be fixed because last time I checked (long time ago)
                // this is not how string equality works in Java.
                if let Value::StringConst(other) = other {
                    *string == *other
                } else {
                    unreachable!()
                }
            }
            Value::ReturnAddress(addr) => {
                if let Value::ReturnAddress(other) = other {
                    *addr == *other
                } else {
                    unreachable!()
                }
            }
            Value::Ref(ptr) => {
                if let Value::Ref(other) = other {
                    ptr == other
                } else {
                    unreachable!()
                }
            }
        }
    }

    fn ne(&self, other: &Self) -> bool { !self.eq(other) }
}

impl Eq for Value {}

impl PartialOrd<Self> for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if !self.is_same_variant(other) {
            return None;
        }

        match self {
            Value::Null => Some(Ordering::Equal),
            Value::Double(double) => {
                if let Value::Double(other) = other {
                    double.partial_cmp(other)
                } else {
                    unreachable!()
                }
            }
            Value::Int(int) => {
                if let Value::Int(other) = other {
                    int.partial_cmp(other)
                } else {
                    unreachable!()
                }
            }
            Value::Short(short) => {
                if let Value::Short(other) = other {
                    short.partial_cmp(other)
                } else {
                    unreachable!()
                }
            }
            Value::Byte(byte) => {
                if let Value::Byte(other) = other {
                    byte.partial_cmp(other)
                } else {
                    unreachable!()
                }
            }
            Value::Long(long) => {
                if let Value::Long(other) = other {
                    long.partial_cmp(other)
                } else {
                    unreachable!()
                }
            }
            Value::Float(float) => {
                if let Value::Float(other) = other {
                    float.partial_cmp(other)
                } else {
                    unreachable!()
                }
            }
            Value::StringConst(string) => {
                //TODO this may need to be fixed because last time I checked (long time ago)
                // this is not how string equality works in Java.
                if let Value::StringConst(other) = other {
                    string.partial_cmp(other)
                } else {
                    unreachable!()
                }
            }
            Value::ReturnAddress(addr) => {
                if let Value::ReturnAddress(other) = other {
                    addr.partial_cmp(other)
                } else {
                    unreachable!()
                }
            }
            Value::Ref(ptr) => {
                if let Value::Ref(other) = other {
                    ptr.partial_cmp(other)
                } else {
                    unreachable!()
                }
            }
        }
    }
}
