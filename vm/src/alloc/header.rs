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

    pub(crate) fn populate_table(&mut self, ptr: *mut u8, fields: Vec<&Field>) {
        let mut next_ptr = ptr;
        // SAFETY: it is valid to index past ptr by VALUE_SIZE because this function
        // does not get called for 0 fields, and nonzero fields means this
        // region of memory is allocated and valid.
        // ptr is aligned at the end of the loop before use.
        let mut end_ptr = unsafe { ptr.add(VALUE_SIZE) };
        let mut i = 0;
        while i < fields.len() {
            let name = fields[i].name.to_string();
            // SAFETY: The invariant that `next_ptr` is always aligned and valid
            // is upheld when initialized above or mutated below.
            unsafe {
                // write the default value to avoid uninitialized memory
                ptr::write(next_ptr as *mut Value, Value::Null);
            }
            self.fields.insert(name, next_ptr as usize - ptr as usize);
            i += 1;

            // avoid UB
            if i != fields.len() {
                let offset = end_ptr.align_offset(VALUE_ALIGN);
                // SAFETY: it is valid to index past ptr by VALUE_SIZE because this function
                // does not get called for 0 fields, and nonzero fields means this
                // region of memory is allocated and valid. Also, we checked for being
                // at the end of the region of mem by checking if we have iterated over all
                // fields.
                unsafe {
                    next_ptr = end_ptr.add(offset);
                    end_ptr = next_ptr.add(VALUE_SIZE);
                }
            }
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
