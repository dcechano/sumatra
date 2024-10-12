use crate::{
    alloc::fields_table::FieldsTable,
    class::Class,
    data_types::{array::ArrayComp, value::Value},
    vm::VM,
};
use core::alloc;
use std::{
    alloc::Layout,
    fmt::{Display, Formatter},
    ptr,
};
use sumatra_parser::{
    constant::Constant,
    desc_types::{FieldType, Primitive},
    field::Field,
    flags::FieldAccessFlags,
};

use super::alloc_type::AllocType;

const ARRAY_CLASS_NAME: &str = "java/lang/Object";

#[derive(Debug)]
pub(crate) struct Header {
    pub class_id: usize,
    pub name: String,
    pub fields: FieldsTable,
    pub array_data: Option<(usize, ArrayComp)>,
}

impl Header {
    #[inline]
    pub(crate) fn new<T: AllocType>(
        class: &Class,
        class_id: usize,
        super_classes: Vec<&'static Class>,
    ) -> (Self, *mut Value) {
        let (ft, values) = FieldsTable::new::<T>(class, super_classes);
        let hdr = Self {
            class_id,
            name: class.get_name(),
            // offsets for the fields cannot be calculated until
            // we put them in. Thus, awkwardly, the Header has to be created
            // and put into memory before we can assemble the fields table.
            fields: ft,
            array_data: None,
        };

        (hdr, values)
    }

    pub(crate) fn new_array(length: usize, array_type: ArrayComp) -> Self {
        Self {
            class_id: 0,
            name: ARRAY_CLASS_NAME.to_string(),
            fields: FieldsTable::default(),
            array_data: Some((length, array_type)),
        }
    }

    pub(crate) unsafe fn populate_table(&mut self, ptr: *mut Value, fields: Vec<&Field>) {
        let mut i = 0;
        while i < fields.len() {
            let name = fields[i].name.to_string();
            // SAFETY: The invariant that `ptr` is always aligned and valid
            // is upheld by the calling method. Additionally, the length of the dynamic
            // array being long enough for this add is also guaranteed by the
            // calling method.
            unsafe {
                let value = if !matches!(fields[i].constant_value, Constant::Dummy) {
                    match &fields[i].constant_value {
                        Constant::Integer(int) => Value::Int(*int),
                        Constant::Float(float) => Value::Float(*float),
                        Constant::Long(long) => Value::Long(*long),
                        Constant::Double(double) => Value::Double(*double),
                        Constant::String(index) => {
                            println!("field name: {}", name);
                            todo!()
                        }
                        invalid => {
                            panic!("Invalid constant_value while initialing obj: {invalid:?}")
                        }
                    }
                } else {
                    match &fields[i].parsed_descriptor.get_field_type() {
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
            self.fields.insert(name, i);
            i += 1;
        }
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let indent = "\t";
        writeln!(f, "Header")?;
        writeln!(f, "{}name: {}", indent, &self.name)?;
        writeln!(f, "{}fields: {:#?}", indent, &self.fields)
    }
}
