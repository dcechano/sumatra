use crate::{
    alloc::fields_table::FieldsTable,
    class::Class,
    data_types::{array::ArrayComp, value::Value},
};
use std::fmt::{Display, Formatter};

use super::alloc_type::AllocType;

const ARRAY_CLASS_NAME: &str = "java/lang/Object";

#[derive(Debug)]
pub struct Header {
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
}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let indent = "\t";
        writeln!(f, "Header")?;
        writeln!(f, "{}name: {}", indent, &self.name)?;
        writeln!(f, "{}fields: {:#?}", indent, &self.fields)
    }
}
