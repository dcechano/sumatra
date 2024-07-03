use std::{
    fmt::{Debug, Formatter},
    ptr,
};

use sumatra_parser::instruction::ArrayType;

use crate::{
    alloc::{alloc_type::NonStatic, oop::HeapAlloc},
    data_types::{hash, value::Value},
};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ArrayRef(*mut HeapAlloc<NonStatic>);

impl ArrayRef {
    /// Create a new array from the given `ArrayType` and `length`.
    pub fn new(length: usize, array_type: ArrayType) -> Self {
        Self(HeapAlloc::new_array(length, array_type))
    }

    /// Returns the `ArrayType` of `ArrayRef` instance.
    pub fn array_type(&self) -> ArrayType {
        // SAFETY: It is safe to dereference the ptr because it is impossible to
        // get an invalid ptr to a HeapAlloc without bypassing the APIs in oop.rs
        // which this binary does not do.
        unsafe { (*self.0).header.array_data.unwrap().1 }
    }

    /// insert the `value` into the array at the given `index`.
    pub fn insert(&mut self, index: usize, value: Value) {
        // SAFETY: It is safe to dereference the ptr because it is impossible to
        // get an invalid ptr to a HeapAlloc without bypassing the APIs in oop.rs
        // which this binary does not do.
        let Some((length, array_type)) = (unsafe { (*self.0).header.array_data.as_ref() }) else {
            panic!("Pointer stored in ArrayRef was not an array!");
        };
        if index >= *length {
            todo!("Throw ArrayIndexOutOfBounds Exception")
        }

        if !Self::validate_type(&value, array_type) {
            panic!("Attempted to put {value:?} into array with type: {array_type:?}");
        }

        unsafe {
            ptr::write((*self.0).elements.add(index), value);
        }
    }

    /// Retrieve the `Value` from the array at the given `index`.
    /// The `ArrayRef` instance still owns the `Value` requested so the returned
    /// `Value` is a clone.
    pub fn get(&self, index: usize) -> Value {
        // SAFETY: It is safe to dereference the ptr because it is impossible to
        // get an invalid ptr to a HeapAlloc without bypassing the APIs in oop.rs
        // which this binary does not do.
        let len = self.len();
        if index >= len {
            todo!("Throw ArrayIndexOutOfBounds Exception")
        }

        unsafe { (*self.0).elements.add(index).as_ref().unwrap().clone() }
    }

    /// Returns all elements of the Java array as a Vec.
    pub fn get_all(&self) -> Vec<Value> {
        (0..self.len())
            .map(|index| self.get(index))
            .collect::<Vec<Value>>()
    }

    /// Get the inner value. Returned as a *const to emphasize that
    /// this return type should NOT be modified except by the GC.
    pub fn get_inner(&self) -> *const HeapAlloc<NonStatic> { self.0 }

    /// Calculate the hash for the ptr backing this array instance
    pub fn hash_code(&self) -> i32 {
        hash(unsafe { std::mem::transmute::<&Self, *const u8>(self) })
    }

    /// Returns the length of the `ArrayRef` instance.
    pub fn len(&self) -> usize {
        // SAFETY: It is safe to dereference the ptr because it is impossible to
        // get an invalid ptr to a HeapAlloc without bypassing the APIs in oop.rs
        // which this binary does not do.
        unsafe {
            let Some((length, _)) = (*self.0).header.array_data.as_ref() else {
                panic!("Pointer stored in ArrayRef was not an array!");
            };
            *length
        }
    }

    /// Validate the `ArrayType` is consistent with the provided `Value`.
    fn validate_type(value: &Value, array_type: &ArrayType) -> bool {
        // `Value::from` cannot be used to convert the `ArrayType` to a `Value`
        // because the `From` impl converts `ArrayType::Ref` to `Value::Null`. Which
        // would cause this function to return `false` in some cases when it should
        // otherwise return `true`.
        match array_type {
            ArrayType::Boolean | ArrayType::Char | ArrayType::Byte => {
                matches!(value, Value::Byte(_))
            }
            ArrayType::Short => matches!(value, Value::Short(_)),
            ArrayType::Int => matches!(value, Value::Int(_)),
            ArrayType::Float => matches!(value, Value::Float(_)),
            ArrayType::Double => matches!(value, Value::Double(_)),
            ArrayType::Long => matches!(value, Value::Long(_)),
            ArrayType::Ref => matches!(value, Value::Ref(_) | Value::Null),
            ArrayType::Dummy => {
                panic!("Invalid ArrayType while validating against a Value.")
            }
        }
    }
}

impl Debug for ArrayRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            let mut debug_tuple = f.debug_tuple("ArrayRef");
            let (len, _) = (*self.0).header.array_data.as_ref().unwrap();
            for i in 0..*len {
                let element = (*self.0).elements.add(i).as_ref().unwrap();
                debug_tuple.field(element);
            }
            debug_tuple.finish()
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    #[cfg(miri)]
    fn test_debug_no_ub() {
        use crate::{alloc::oop::HeapAlloc, reference_types::ArrayRef, value::Value};
        use sumatra_parser::instruction::ArrayType;

        const LENGTH: usize = 3;
        const ARRAY_TYPE: ArrayType = ArrayType::Int;

        let mut array = ArrayRef::new(LENGTH, ARRAY_TYPE);
        array.insert(0, Value::Int(0));
        array.insert(1, Value::Int(1));
        array.insert(2, Value::Int(2));

        let debug_string = "ArrayRef(Int(0), Int(1), Int(2))".to_string();
        assert_eq!(format!("{array:?}"), debug_string);
        unsafe {
            HeapAlloc::dealloc_test_obj(array.0);
        }
    }
}
