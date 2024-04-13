use std::{
    alloc,
    alloc::{handle_alloc_error, Layout},
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr,
};

use anyhow::{bail, Result};

use sumatra_parser::{field::Field, flags::FieldAccessFlags};

use crate::{class::Class, value::Value};

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
    pub name: String,
    pub fields: FieldsTable,
}

impl Header {
    #[inline]
    fn new(class: &Class, class_index: usize) -> Self {
        Self {
            class_index,
            name: class.get_name(),
            // offsets for the fields cannot be calculated until
            // we put them in. Thus, awkwardly, the Header has to be created
            // and put into memory before we can assemble the fields table.
            fields: FieldsTable::with_capacity(class.fields.len()),
        }
    }

    fn populate_table(&mut self, ptr: *mut u8, fields: Vec<&Field>, alignment: usize) {
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
                let offset = end_ptr.align_offset(alignment);
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

#[repr(C)]
pub(crate) struct HeapAlloc<'data> {
    pub header: Header,
    pub data: *mut u8,
    _phantom: PhantomData<&'data [u8]>,
}

impl<'data> HeapAlloc<'data> {
    #[allow(clippy::new_ret_no_self)]
    #[inline]
    pub(crate) fn new(class: &Class, index: usize) -> *mut u8 {
        Self::new_inner(class, index, false)
    }

    #[inline]
    pub(crate) fn new_static(class: &Class, index: usize) -> *mut u8 {
        Self::new_inner(class, index, true)
    }

    #[inline]
    fn new_inner(class: &Class, index: usize, statik: bool) -> *mut u8 {
        // SAFETY: `Layout::new::<HeapAlloc>())` is valid so alloc is safe.
        let ptr = unsafe { alloc::alloc(Layout::new::<HeapAlloc>()) };
        if ptr.is_null() {
            handle_alloc_error(Layout::new::<HeapAlloc>())
        }

        let fields = match statik {
            true => class
                .fields
                .values()
                .filter(|v| v.access_flags.contains(FieldAccessFlags::STATIC))
                .collect::<Vec<&Field>>(),
            false => class
                .fields
                .values()
                .filter(|v| !v.access_flags.contains(FieldAccessFlags::STATIC))
                .collect::<Vec<&Field>>(),
        };
        let mut header = Header::new(class, index);
        // ptr now allocated
        // TODO consider converting to match statement for consistency with code below
        let data = if !fields.is_empty() {
            // SAFETY: since fields len is non 0, alloc is safe.
            unsafe { alloc::alloc(Layout::array::<Value>(fields.len()).unwrap()) }
        } else {
            ptr::null_mut()
        };
        // finish header by populating the offset table
        if !data.is_null() {
            header.populate_table(data, fields, VALUE_ALIGN);
        }

        // SAFETY: ptr is valid for writes since we asserted nonnull above.
        unsafe {
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
    pub(crate) fn get_field(&self, name: &str) -> Result<&Value> {
        // SAFETY: If the offset is valid the area of memory is valid
        // since offset is calculated with respect to the area of memory.
        unsafe {
            let value = self.get_field_inner(name)? as *const Value;
            Ok(&*value)
        }
    }

    #[inline]
    fn get_field_inner(&self, name: &str) -> Result<*mut u8> {
        let offset = match self.header.fields.get(name) {
            None => {
                bail!("No field with name: {name}");
            }
            Some(offset) => offset,
        };
        // SAFETY: offset is valid due to the offset being calculated from the
        // memory region itself, so the offset always points into valid memory.
        unsafe { Ok(self.data.add(*offset)) }
    }

    #[inline]
    pub(crate) fn set_field(&mut self, name: &str, data: Value) -> Result<()> {
        let field = self.get_field_inner(name)?;
        // SAFETY: The validity of the ptr is upheld by the get_field_inner method.
        unsafe {
            ptr::write(field as *mut Value, data);
        }
        Ok(())
    }

    #[inline]
    unsafe fn deallocate(heap: *mut HeapAlloc) {
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
        writeln!(f, "{}name: {}", indent, &self.name)?;
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
    use std::{fmt::Debug, ptr};

    use sumatra_parser::{class_file::ClassFile, flags::FieldAccessFlags};

    use crate::{class::Class, oop::HeapAlloc, value::Value};

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
            let ptr = HeapAlloc::new(&Class::from(&class_file), 0);
            unsafe {
                let heap = &mut *(ptr as *mut HeapAlloc);
                for field in class_file.fields {
                    if !field.access_flags.contains(FieldAccessFlags::STATIC) {
                        heap.set_field(&field.name, Value::Long(69420)).unwrap();
                    }
                }

                HeapAlloc::deallocate(ptr as *mut HeapAlloc);
            }
        }
    }

    //FIXME: Since the `printWriter` field is static it does not get added to the
    // fields which was not true when test was written. Ignore for now until a fix
    // is made. Perhaps test should be moved for the method area where the
    // static fields will exist.
    #[test]
    #[ignore]
    fn alloc2() {
        let taco = CLASSES[4];

        let class_file = ClassFile::parse_class(taco).unwrap();
        let field = ClassFile::parse_class(taco).unwrap();

        let containing_class = HeapAlloc::new(&Class::from(&class_file), 0);
        let field_ref = HeapAlloc::new(&Class::from(&field), 0);

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
}
