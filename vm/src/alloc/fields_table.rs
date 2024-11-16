use std::{
    alloc::{self, Layout},
    collections::HashMap,
    fmt::{Debug, Formatter},
    ops::{Deref, DerefMut},
    ptr,
};

use sumatra_parser::{
    constant::Constant,
    desc_types::{FieldType, Primitive},
    field::Field,
    flags::FieldAccessFlags,
};

use crate::{class::Class, data_types::value::Value, vm::VM};

use super::alloc_type::AllocType;

#[derive(Default)]
#[repr(transparent)]
pub(crate) struct FieldsTable(HashMap<String, usize>);

impl FieldsTable {
    pub(crate) fn new<T: AllocType>(
        class: &Class,
        super_classes: Vec<&'static Class>,
    ) -> (Self, *mut Value) {
        let class_n_fields = Self::filter_fields::<T>(class, super_classes);
        let mut ft = Self(HashMap::with_capacity(class_n_fields.len()));
        let ptr = Self::alloc_table_vec(class_n_fields.len());

        if ptr.is_null() {
            return (ft, ptr);
        }

        let mut i = 0;
        while i < class_n_fields.len() {
            let (class, field) = class_n_fields[i];
            let name = field.name.to_string();
            // SAFETY: The invariant that `ptr` is always aligned and valid
            // is upheld by Self::alloc_table_vec. Additionally, the length of the dynamic
            // array being long enough for this add is also guaranteed by the
            // same.
            unsafe {
                let value = if !matches!(field.constant_value, Constant::Dummy) {
                    match &field.constant_value {
                        Constant::Integer(int) => Value::Int(*int),
                        Constant::Float(float) => Value::Float(*float),
                        Constant::Long(long) => Value::Long(*long),
                        Constant::Double(double) => Value::Double(*double),
                        Constant::String(index) => {
                            let string = class.cp.get_utf8(*index).unwrap();
                            Value::StringConst(string.to_string())
                        }
                        invalid => {
                            panic!("Invalid constant_value while initialing obj: {invalid:?}")
                        }
                    }
                } else {
                    match &field.parsed_descriptor.get_field_type() {
                        FieldType::Base(primitive) => match primitive {
                            Primitive::Byte => Value::Byte(0),
                            Primitive::Char | Primitive::Int => Value::Int(0),
                            Primitive::Double => Value::Double(0.0),
                            Primitive::Float => Value::Float(0.0),
                            Primitive::Long => Value::Long(0),
                            Primitive::Short => Value::Short(0),
                            Primitive::Boolean => Value::Int(0),
                            Primitive::Invalid => panic!("Invalid primitive in field descriptor"),
                        },
                        FieldType::Invalid => panic!("Invalid field descriptor!"),
                        _ => Value::Null,
                    }
                };

                // write the default value to avoid uninitialized memory
                ptr::write(ptr.add(i), value);
            }
            ft.insert(name, i);
            i += 1;
        }
        (ft, ptr)
    }

    fn alloc_table_vec<'f>(len: usize) -> *mut Value {
        match len > 0 {
            // SAFETY: since fields len is non 0, alloc is safe.
            true => unsafe { alloc::alloc(Layout::array::<Value>(len).unwrap()) as *mut Value },
            false => ptr::null_mut(),
        }
    }

    fn filter_fields<'f, T: AllocType>(
        class: &'f Class,
        super_classes: Vec<&'static Class>,
    ) -> Vec<(&'f Class, &'f Field)> {
        match T::is_static() {
            true => class
                .fields
                .values()
                .filter(|v| v.access_flags.contains(FieldAccessFlags::STATIC))
                .map(|f| (class, f))
                .collect::<Vec<(&'f Class, &'f Field)>>(),
            false => {
                let mut primary_fields = class
                    .fields
                    .values()
                    .filter(|v| !v.access_flags.contains(FieldAccessFlags::STATIC))
                    .map(|f| (class, f))
                    .collect::<Vec<(&'f Class, &'f Field)>>();
                let ancestor_fields = super_classes.into_iter().flat_map(|class| {
                    class
                        .fields
                        .values()
                        .filter(|v| !v.access_flags.contains(FieldAccessFlags::STATIC))
                        .map(move |f| (class, f))
                });
                primary_fields.extend(ancestor_fields);
                primary_fields
            }
        }
    }
}

impl Deref for FieldsTable {
    type Target = HashMap<String, usize>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for FieldsTable {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Debug for FieldsTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { writeln!(f, "{:#?}", &self.0) }
}
