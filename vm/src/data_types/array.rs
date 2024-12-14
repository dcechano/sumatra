use std::{
    fmt::{Debug, Formatter},
    ptr,
    str::FromStr,
};
use sumatra_parser::{
    desc_types,
    desc_types::{FieldType, Primitive},
    instruction::ArrayType,
};

use crate::{
    alloc::{alloc_type::NonStatic, oop::HeapAlloc},
    data_types::value::Value,
    result::{Error, Result},
    vm, vm_error,
};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ArrayRef(*mut HeapAlloc<NonStatic>);

impl ArrayRef {
    /// Create a new array from the given `ArrayType` and `length`.
    pub fn new(length: usize, array_type: ArrayComp) -> Self {
        Self(HeapAlloc::new_array(length, array_type))
    }

    /// Returns the `ArrayComp` of `ArrayRef` instance.
    pub fn array_comp(&self) -> &ArrayComp {
        // SAFETY: It is safe to dereference the ptr because it is impossible to
        // get an invalid ptr to a HeapAlloc without bypassing the APIs in oop.rs
        // which this binary does not do.
        unsafe { &(*self.0).header.array_data.as_ref().unwrap().1 }
    }

    /// insert the `value` into the array at the given `index`. Does superficial
    /// checking of the component types but for reference types the checking
    /// should be done by the caller.
    pub fn insert(&mut self, index: usize, value: Value) {
        // SAFETY: It is safe to dereference the ptr because it is impossible to
        // get an invalid ptr to a HeapAlloc without bypassing the APIs in oop.rs
        // which this binary does not do.
        let Some((length, array_comp)) = (unsafe { (*self.0).header.array_data.as_ref() }) else {
            panic!("Pointer stored in ArrayRef was not an array!");
        };
        if index >= *length {
            todo!("Throw ArrayIndexOutOfBounds Exception")
        }

        if !Self::validate_comp(&value, array_comp) {
            panic!("Attempted to put {value:?} into array with type: {array_comp:?}");
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

    /// Calculate the hash for the ptr backing this object instance
    pub fn hash_code(&self) -> i32 {
        let ptr = self.0 as usize;
        let u_32 = (ptr & u32::MAX as usize) as u32;
        unsafe { std::mem::transmute::<u32, i32>(u_32) }
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

    /// Validate the `ArrayComp` is consistent with the provided `Value`.
    /// Does not compare the classes of reference types.
    fn validate_comp(value: &Value, array_type: &ArrayComp) -> bool {
        // `Value::from` cannot be used to convert the `ArrayComp` to a `Value`
        // because the `From` impl converts `ArrayComp::Ref` to `Value::Null`. Which
        // would cause this function to return `false` in some cases when it should
        // otherwise return `true`.
        match array_type {
            ArrayComp::Boolean | ArrayComp::Char | ArrayComp::Byte => {
                matches!(value, Value::Byte(_))
            }
            ArrayComp::Short => matches!(value, Value::Short(_)),
            ArrayComp::Int => matches!(value, Value::Int(_)),
            ArrayComp::Float => matches!(value, Value::Float(_)),
            ArrayComp::Double => matches!(value, Value::Double(_)),
            ArrayComp::Long => matches!(value, Value::Long(_)),
            //TODO consider if the value below is correct. It does not consider the value
            // of the string in ArrayComp::Ref
            ArrayComp::Array(_) | ArrayComp::Class(_) => {
                matches!(value, Value::Ref(_) | Value::Null)
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

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum ArrayComp {
    Byte,
    Char,
    Class(String),
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
    Array(Box<ArrayComp>),
}

impl ArrayComp {
    pub fn dimension(&self) -> usize {
        let mut dim = 1;
        let mut comp = self;
        while let ArrayComp::Array(inner) = comp {
            dim += 1;
            comp = inner;
        }
        dim
    }

    pub fn root_comp(&self) -> &ArrayComp {
        let mut comp = self;
        while let ArrayComp::Array(inner) = comp {
            comp = inner;
        }
        comp
    }

    pub fn prim_array(dim: usize, prim: Primitive) -> Self {
        if dim < 1 {
            panic!("Cannot initialize an array without at least 1 dimension.");
        }

        if dim == 1 {
            return ArrayComp::from(prim);
        }

        let mut temp = Box::new(ArrayComp::from(prim));
        for _ in 0..dim - 1 {
            temp = Box::new(ArrayComp::Array(temp));
        }
        *temp
    }

    pub fn class_array(dim: usize, name: String) -> Self {
        if dim < 1 {
            panic!("Cannot initialize an array without at least 1 dimension.");
        }

        if dim == 1 {
            return ArrayComp::Class(name);
        }

        let mut temp = Box::new(ArrayComp::Class(name));
        for _ in 0..dim - 1 {
            temp = Box::new(ArrayComp::Array(temp));
        }
        *temp
    }
}

impl FromStr for ArrayComp {
    type Err = Error;

    fn from_str(array_desc: &str) -> Result<Self> {
        // Field Descriptors and Array descriptors are explicitly the same thing.
        // Thus, reusing code.
        //https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-FieldType
        let field_type = array_desc.parse::<FieldType>().map_err(|e| {
            log::error!(
                "Error encountered while parsing the FieldType \
                    from an array descriptor: {e:?} "
            );
            Error::ClassValidation
        })?;
        let array_comp = match field_type {
            FieldType::Base(primitive) => ArrayComp::from(primitive),
            FieldType::Object(class) => ArrayComp::Class(class),
            FieldType::Array(typ, dim) => match typ {
                desc_types::ArrayType::Object(class) => ArrayComp::class_array(dim, class),
                desc_types::ArrayType::Primitive(prim) => ArrayComp::prim_array(dim, prim),
            },
            FieldType::Invalid => panic!("Invalid primitive when parsing to ArrayComp."),
        };
        Ok(array_comp)
    }
}

impl From<Primitive> for ArrayComp {
    fn from(prim: Primitive) -> Self {
        match prim {
            Primitive::Byte => ArrayComp::Byte,
            Primitive::Char => ArrayComp::Char,
            Primitive::Double => ArrayComp::Double,
            Primitive::Float => ArrayComp::Float,
            Primitive::Int => ArrayComp::Int,
            Primitive::Long => ArrayComp::Long,
            Primitive::Short => ArrayComp::Short,
            Primitive::Boolean => ArrayComp::Boolean,
            Primitive::Invalid => panic!("Invalid primitive when parsing to ArrayComp."),
        }
    }
}

impl TryFrom<ArrayType> for ArrayComp {
    type Error = Error;

    fn try_from(array_type: ArrayType) -> Result<Self> {
        let comp = match array_type {
            ArrayType::Boolean => ArrayComp::Boolean,
            ArrayType::Char => ArrayComp::Char,
            ArrayType::Float => ArrayComp::Float,
            ArrayType::Double => ArrayComp::Double,
            ArrayType::Byte => ArrayComp::Byte,
            ArrayType::Short => ArrayComp::Short,
            ArrayType::Int => ArrayComp::Int,
            ArrayType::Long => ArrayComp::Long,
            ArrayType::Dummy => vm_error!("Tried to convert from ArrayType::Dummy to ArrayComp"),
            ArrayType::Ref => vm_error!("Tried to convert from ArrayType::Ref to ArrayComp::Array(_) or ArrayComp::Class(_)"),
        };
        Ok(comp)
    }
}

#[cfg(test)]
mod tests {
    use crate::data_types::array::ArrayComp;
    use sumatra_parser::desc_types::Primitive;

    #[test]
    #[cfg(miri)]
    fn test_debug_no_ub() {
        use crate::{
            alloc::oop::HeapAlloc,
            data_types::{array::ArrayRef, value::Value},
        };

        const LENGTH: usize = 3;
        const ARRAY_TYPE: ArrayComp = ArrayComp::Int;

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

    #[test]
    fn test_array_comp_prim() {
        let array = ArrayComp::prim_array(3, Primitive::Int);
        assert_eq!(
            array,
            ArrayComp::Array(Box::new(ArrayComp::Array(Box::new(ArrayComp::Int))))
        );

        let array = ArrayComp::prim_array(1, Primitive::Int);
        assert_eq!(array, ArrayComp::Int)
    }

    #[test]
    fn test_array_comp_class() {
        let array = ArrayComp::class_array(5, "java/lang/Object".to_string());
        assert_eq!(
            array,
            ArrayComp::Array(Box::new(ArrayComp::Array(Box::new(ArrayComp::Array(
                Box::new(ArrayComp::Array(Box::new(ArrayComp::Class(
                    "java/lang/Object".to_string()
                ))))
            )))))
        )
    }

    #[test]
    fn test_array_dimension() {
        let dim = ArrayComp::class_array(5, "java/lang/Object".to_string()).dimension();
        assert_eq!(dim, 5);

        let dim = ArrayComp::prim_array(3, Primitive::Int).dimension();
        assert_eq!(dim, 3);

        let dim = ArrayComp::prim_array(1, Primitive::Int).dimension();
        assert_eq!(dim, 1)
    }

    #[test]
    fn test_root_comp() {
        let array = ArrayComp::class_array(5, "java/lang/Object".to_string());
        assert_eq!(
            array.root_comp(),
            &ArrayComp::Class("java/lang/Object".to_string())
        );

        let array = ArrayComp::prim_array(42, Primitive::Byte);
        assert_eq!(array.root_comp(), &ArrayComp::Byte);
    }

    #[test]
    fn test_parse_array() {
        let array = "[[[[I".parse::<ArrayComp>().unwrap();
        assert_eq!(array, ArrayComp::prim_array(4, Primitive::Int));

        // The reason we leave a trailing ';' when parsing and not when creating the
        // array directly with ArrayComp::class_array is that the class string
        // will have a trailing ';' when loaded from the constant pool. The
        // parse method expects this and will fill otherwise.
        let array = "[[[[[[[Ljava/lang/Object;".parse::<ArrayComp>().unwrap();
        assert_eq!(
            array,
            ArrayComp::class_array(7, "java/lang/Object".to_string())
        );
    }
}
