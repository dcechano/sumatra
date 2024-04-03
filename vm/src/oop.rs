use std::{
    alloc,
    alloc::Layout,
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr,
};

use anyhow::{bail, Result};

use sumatra_parser::{class_file::ClassFile, field::Field};

use crate::value::Value;

const VALUE_SIZE: usize = mem::size_of::<Value>();
const VALUE_ALIGN: usize = mem::align_of::<Value>();

#[derive(Default)]
#[repr(transparent)]
pub(crate) struct FieldsTable(HashMap<String, usize>);

impl FieldsTable {
    #[inline]
    fn with_capacity(cap: usize) -> Self { Self(HashMap::with_capacity(cap)) }
}

impl Deref for FieldsTable {
    type Target = HashMap<String, usize>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for FieldsTable {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct Header {
    pub class_index: usize,
    pub source_file: String,
    pub fields: FieldsTable,
}

impl Header {
    #[inline]
    fn new(class: &ClassFile, class_index: usize) -> Self {
        Self {
            class_index,
            source_file: class
                .get_utf8(class.attributes.source_file.0)
                .unwrap()
                .to_string(),
            // offsets for the fields cannot be calculated until
            // we put them in. Thus, awkwardly, the Header has to be created
            // and put into memory before we can assemble the fields table.
            fields: FieldsTable::with_capacity(class.fields.len()),
        }
    }

    fn populate_table(&mut self, ptr: *mut u8, fields: &[Field], alignment: usize) {
        unsafe {
            let mut next_ptr = ptr;
            let mut end_ptr = ptr.add(VALUE_SIZE);
            let mut i = 0;
            while i < fields.len() {
                let name = fields[i].name.to_string();
                // write the default value to avoid uninitialized memory
                ptr::write(next_ptr as *mut Value, Value::Null);
                self.fields.insert(name, next_ptr as usize - ptr as usize);
                i += 1;

                // avoid UB
                if i != fields.len() {
                    let offset = end_ptr.align_offset(alignment);
                    next_ptr = end_ptr.add(offset);
                    end_ptr = next_ptr.add(VALUE_SIZE);
                }
            }
        }
    }
}

#[repr(C)]
pub(crate) struct HeapAlloc<'data> {
    pub header: Header,
    pub data: *mut u8,
    _phantom: PhantomData<&'data [u8]>,
}

impl<'data> HeapAlloc<'data> {
    #[allow(clippy::new_ret_no_self)]
    pub(crate) fn new(class: &ClassFile, index: usize) -> *mut u8 {
        unsafe {
            let ptr = alloc::alloc(Layout::new::<HeapAlloc>());

            let num_fields = class.fields.len();
            let mut header = Header::new(class, index);
            // ptr now allocated
            let data = if num_fields != 0 {
                alloc::alloc(Layout::array::<Value>(num_fields).unwrap())
            } else {
                ptr::null_mut()
            };
            // finish header by populating the offset table
            if !data.is_null() {
                header.populate_table(data, &class.fields, VALUE_ALIGN);
            }

            ptr::write(
                ptr as *mut HeapAlloc,
                HeapAlloc {
                    header,
                    data,
                    _phantom: Default::default(),
                },
            );
            ptr
        }
    }

    #[inline]
    pub(crate) fn get_field(&self, name: &str) -> Result<Value> {
        unsafe {
            let value = ptr::read(self.get_field_inner(name)? as *const Value);
            Ok(value)
        }
    }

    #[inline]
    pub(crate) unsafe fn get_field_inner(&self, name: &str) -> Result<*mut u8> {
        let offset = match self.header.fields.get(name) {
            None => {
                bail!("No field with name: {name}");
            }
            Some(offset) => offset,
        };

        Ok(self.data.add(*offset))
    }

    #[inline]
    pub(crate) fn set_field(&mut self, name: &str, data: Value) -> Result<()> {
        unsafe {
            let field = self.get_field_inner(name)?;
            ptr::write(field as *mut Value, data);
        }
        Ok(())
    }

    #[inline]
    pub(crate) unsafe fn deallocate(heap: *mut HeapAlloc) {
        if heap.is_null() {
            return;
        }
        let data = (*heap).data;
        if !data.is_null() {
            // We need not worry about fields.len == 0 because the only way the ptr
            // is not null is that there were fields to justify the initial allocation.
            let size = (*heap).header.fields.len();
            debug_assert!(size != 0);
            let layout = Layout::array::<Value>(size).unwrap();
            // Self::dealloc_data(data, size);
            alloc::dealloc(data, layout);
        }

        ptr::drop_in_place(&mut (*heap).header as *mut Header);

        alloc::dealloc(heap as *mut u8, Layout::new::<HeapAlloc>());
    }

    // #[inline]
    // pub(crate) unsafe fn dealloc_data(data: *mut u8, size: usize) {
    //
    // }
}

impl<'data> Debug for HeapAlloc<'data> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self) }
}

impl Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let indent = "\t";
        writeln!(f, "Header")?;
        writeln!(f, "{}source file: {}", indent, &self.source_file)?;
        writeln!(f, "{}fields: {:#?}", indent, &self.fields)
    }
}

impl<'data> Display for HeapAlloc<'data> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let indent = "\t";
        writeln!(f, "HeapObj")?;
        writeln!(f, "{indent}{}", self.header)?;
        for (name, offset) in self.header.fields.iter() {
            // SAFETY: It is ok deref ptr here because the only way for it to
            // be null is for the class to have 0 fields.
            unsafe {
                let ptr = self.data.add(*offset) as *const Value;
                writeln!(f, "{name} {:?}", *ptr)?;
            }
        }
        write!(f, "")
    }
}

impl Debug for FieldsTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { writeln!(f, "{:#?}", &self.0) }
}

#[cfg(test)]
mod test {
    use crate::{oop::HeapAlloc, value::Value};
    use std::ptr;
    use sumatra_parser::class_file::ClassFile;

    const CLASSES: [&str; 6] = [
        "/home/dylan/Documents/RustProjects/jheap/java/target/production/java/Main.class",
        "/home/dylan/Documents/RustProjects/jheap/java/target/production/java/Interface.class",
        "/home/dylan/Documents/RustProjects/jheap/java/target/production/java/Import.class",
        "/home/dylan/Documents/RustProjects/jheap/java/target/production/java/Simple.class",
        "/home/dylan/Documents/RustProjects/jheap/java/target/production/java/Taco.class",
        "/home/dylan/Documents/RustProjects/jheap/java/target/production/java/Fields.class",
    ];

    #[test]
    fn alloc() {
        for class in CLASSES {
            let class_file = ClassFile::parse_class(class).unwrap();
            let ptr = HeapAlloc::new(&class_file, 0);
            unsafe {
                let heap = &mut *(ptr as *mut HeapAlloc);
                for field in class_file.fields {
                    heap.set_field(&field.name, Value::Long(69420)).unwrap();
                }

                HeapAlloc::deallocate(ptr as *mut HeapAlloc);
            }
        }
    }

    #[test]
    fn alloc2() {
        let taco = CLASSES[4];

        let class_file = ClassFile::parse_class(taco).unwrap();
        let field = ClassFile::parse_class(taco).unwrap();

        let containing_class = HeapAlloc::new(&class_file, 0);
        let field_ref = HeapAlloc::new(&field, 0);

        unsafe {
            let heap = &mut *(containing_class as *mut HeapAlloc);
            heap.set_field(
                "printWriter",
                Value::RefType(ptr::read(field_ref as *const HeapAlloc)),
            )
            .unwrap();

            HeapAlloc::deallocate(field_ref as *mut HeapAlloc);
            HeapAlloc::deallocate(containing_class as *mut HeapAlloc);
        }
    }

    #[test]
    fn miri() {}
}
