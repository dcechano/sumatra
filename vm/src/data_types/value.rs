use std::cmp::Ordering;

use sumatra_parser::instruction::ArrayType;

use crate::data_types::{
    array::{ArrayComp, ArrayRef},
    object::ObjRef,
};

#[derive(Default, Debug, Clone)]
pub enum Value {
    #[default]
    Null,
    Byte(i8),
    Double(f64),
    Dynamic {
        bootstrap_method_attr_index: usize,
        name_and_type_index: usize,
    },
    Float(f32),
    Int(i32),
    Long(i64),
    MethodHandle {
        reference_kind: u8,
        reference_index: usize,
    },
    MethodType(usize),
    ReturnAddress(usize),
    Ref(RefType),
    Short(i16),
    StringConst(String),
}

impl Value {
    pub fn default_vec(cap: usize) -> Vec<Value> { vec![Value::Null; cap] }

    /// Allocates a new Java Obj and returns Value::Ref for the new object.
    pub fn new_object(obj: ObjRef) -> Value { Value::Ref(RefType::Object(obj)) }

    /// Allocates a new Java array and returns Value::Ref for the new array.
    pub(crate) fn new_array(array_ref: ArrayRef) -> Value { Value::Ref(RefType::Array(array_ref)) }

    pub fn populate_locals(num_locals: usize, params: &mut Vec<Value>) {
        if params.len() > num_locals {
            panic!("The number of locals cannot be the greater than the number of params.");
        }

        let num_dummies = num_locals - params.len();
        params.extend(vec![Value::Null; num_dummies]);
    }

    pub(crate) fn is_same_variant(&self, other: &Value) -> bool {
        match self {
            Value::Null => matches!(other, Value::Null),
            Value::Byte(_) => matches!(other, Value::Byte(_)),
            Value::Double(_) => matches!(other, Value::Double(_)),
            Value::Dynamic { .. } => matches!(other, Value::Dynamic { .. }),
            Value::Float(_) => matches!(other, Value::Float(_)),
            Value::Int(_) => matches!(other, Value::Int(_)),
            Value::Long(_) => matches!(other, Value::Long(_)),
            Value::Ref(_) => matches!(other, Value::Ref(_)),
            Value::ReturnAddress(_) => matches!(other, Value::ReturnAddress(_)),
            Value::Short(_) => matches!(other, Value::Short(_)),
            Value::StringConst(_) => matches!(other, Value::StringConst(_)),
            Value::MethodHandle { .. } => matches!(other, Value::MethodHandle { .. }),
            Value::MethodType(_) => matches!(other, Value::MethodType(_)),
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
            _ => {
                panic!("Incomparable values: {:?} and {:?}", self, other);
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
            // This is fine because `other` is verified to be also Value::Null above.
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
            _ => {
                panic!("Incomparable values: {:?} and {:?}", self, other);
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum RefType {
    Object(ObjRef),
    Array(ArrayRef),
}

impl From<ArrayComp> for Value {
    fn from(array_comp: ArrayComp) -> Self {
        match array_comp {
            ArrayComp::Boolean
            | ArrayComp::Char
            | ArrayComp::Short
            | ArrayComp::Byte
            | ArrayComp::Int => Value::Int(0),
            ArrayComp::Float => Value::Float(0.0),
            ArrayComp::Double => Value::Double(0.0),
            ArrayComp::Long => Value::Long(0),
            ArrayComp::Ref(_) => Value::Null,
        }
    }
}
