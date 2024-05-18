use std::{
    fmt::{Display, Formatter},
    ptr,
};

use sumatra_parser::field::Field;

use crate::{
    alloc::{fields_table::FieldsTable, VALUE_ALIGN, VALUE_SIZE},
    class::Class,
    value::Value,
};

#[derive(Debug)]
pub(crate) struct Header {
    pub class_index: usize,
    pub name: String,
    pub fields: FieldsTable,
}

impl Header {
    #[inline]
    pub(crate) fn new(class: &Class, class_index: usize) -> Self {
        Self {
            class_index,
            name: class.get_name(),
            // offsets for the fields cannot be calculated until
            // we put them in. Thus, awkwardly, the Header has to be created
            // and put into memory before we can assemble the fields table.
            fields: FieldsTable::with_capacity(class.fields.len()),
        }
    }

    pub(crate) unsafe fn populate_table(&mut self, ptr: *mut Value, fields: Vec<&Field>) {
        let mut i = 0;
        while i < fields.len() {
            let name = fields[i].name.to_string();
            // SAFETY: The invariant that `ptr` is always aligned and valid
            // is upheld by the calling method. Additionally, the length of the dynamic array
            // being long enough for this add is also guaranteed by the calling method.
            unsafe {
                // write the default value to avoid uninitialized memory
                ptr::write(ptr.add(i), Value::Null);
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
