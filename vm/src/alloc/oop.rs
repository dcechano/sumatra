use std::{
    alloc,
    alloc::{handle_alloc_error, Layout},
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr,
};

use anyhow::{bail, Result};

use sumatra_parser::{field::Field, flags::FieldAccessFlags};

use crate::{
    alloc::{
        alloc_type::{AllocType, NonStatic, Static},
        header::Header,
    },
    class::Class,
    value::Value,
};

pub(crate) struct HeapAlloc<T: AllocType> {
    pub header: Header,
    pub data: *mut u8,
    _phantom: PhantomData<[u8]>,
    _static: PhantomData<T>,
}

impl<T: AllocType> HeapAlloc<T> {
    #[inline]
    fn new_inner(class: &Class, index: usize) -> (Header, *mut u8) {
        let fields = match T::is_static() {
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
        let data = match !fields.is_empty() {
            // SAFETY: since fields len is non 0, alloc is safe.
            true => unsafe { alloc::alloc(Layout::array::<Value>(fields.len()).unwrap()) },
            false => ptr::null_mut(),
        };
        // finish header by populating the offset table
        if !data.is_null() {
            header.populate_table(data, fields);
        }
        (header, data)
    }

    #[inline]
    pub(crate) fn get_field(&self, name: &str) -> Result<&'static Value> {
        // SAFETY: If the offset is valid the area of memory is valid
        // since offset is calculated with respect to the area of memory.
        unsafe {
            let value = self.get_field_inner(name)? as *const Value;
            Ok(&*value)
        }
    }

    #[inline]
    pub(crate) fn get_field_mut(&self, name: &str) -> Result<&'static mut Value> {
        // SAFETY: If the offset is valid the area of memory is valid
        // since offset is calculated with respect to the area of memory.
        unsafe {
            let value = self.get_field_inner(name)? as *mut Value;
            Ok(&mut *value)
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
    unsafe fn deallocate(heap: *mut HeapAlloc<T>) {
        // This function should only be called on a pointer to a HeapAlloc<NonStatic>;
        debug_assert!(
            !T::is_static(),
            "HeapAlloc::deallocate should not be passed a pointer \
            to a HeapAlloc<Static>."
        );
        if heap.is_null() {
            return;
        }

        Self::dealloc_data(heap);

        ptr::drop_in_place(&mut (*heap).header as *mut Header);

        alloc::dealloc(heap as *mut u8, Layout::new::<HeapAlloc<NonStatic>>());
    }

    #[inline]
    unsafe fn dealloc_data(heap: *mut HeapAlloc<T>) {
        let data = (*heap).data;
        if !data.is_null() {
            // We need not worry about fields.len == 0 because the only way the ptr
            // is not null is that there were fields to justify the initial allocation.
            let size = (*heap).header.fields.len();
            debug_assert!(size != 0);
            let layout = Layout::array::<Value>(size).unwrap();
            alloc::dealloc(data, layout);
        }
    }
}

impl HeapAlloc<Static> {
    #[inline]
    pub(crate) fn new(class: &Class, index: usize) -> HeapAlloc<Static> {
        let (header, data) = Self::new_inner(class, index);
        HeapAlloc {
            header,
            data,
            _phantom: Default::default(),
            _static: Default::default(),
        }
    }
}

impl HeapAlloc<NonStatic> {
    #[allow(clippy::new_ret_no_self)]
    #[inline]
    pub(crate) fn new(class: &Class, index: usize) -> *mut u8 {
        // SAFETY: `Layout::new::<HeapAlloc>())` is valid so alloc is safe.
        let ptr = unsafe { alloc::alloc(Layout::new::<HeapAlloc<NonStatic>>()) };
        if ptr.is_null() {
            handle_alloc_error(Layout::new::<HeapAlloc<NonStatic>>())
        }

        let (header, data) = Self::new_inner(class, index);
        // SAFETY: ptr is valid for writes since we asserted nonnull above.
        unsafe {
            ptr::write(
                ptr as *mut HeapAlloc<NonStatic>,
                HeapAlloc {
                    header,
                    data,
                    _phantom: Default::default(),
                    _static: Default::default(),
                },
            );
            ptr
        }
    }
}

impl<T: AllocType> Drop for HeapAlloc<T> {
    fn drop(&mut self) {
        /*
            If Self is HeapAlloc<Static> then Self was stack allocated and not
            manually allocated as HeapAlloc<NonStatic> would have been.
            As a result, HeapAlloc<Static> simply deallocates its inner data buffer
            (which is always manually allocated), then simply allows itself to drop.
            HeapAlloc<NonStatic> requires more business logic.
        */
        unsafe {
            match T::is_static() {
                true => {
                    HeapAlloc::dealloc_data(self as *mut Self);
                }
                false => {
                    Self::deallocate(self as *mut Self);
                }
            }
        }
    }
}

impl<T: AllocType> Debug for HeapAlloc<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self) }
}

impl<T: AllocType> Display for HeapAlloc<T> {
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

#[cfg(test)]
mod test {
    use std::fmt::Debug;

    use sumatra_parser::{class_file::ClassFile, flags::FieldAccessFlags};

    use crate::{
        alloc::oop::{HeapAlloc, NonStatic},
        class::Class,
        value::Value,
    };

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
            let ptr = HeapAlloc::<NonStatic>::new(&Class::from(&class_file), 0);
            unsafe {
                let heap = &mut *(ptr as *mut HeapAlloc<NonStatic>);
                for field in class_file.fields {
                    if !field.access_flags.contains(FieldAccessFlags::STATIC) {
                        heap.set_field(&field.name, Value::Long(69420)).unwrap();
                    }
                }

                HeapAlloc::deallocate(ptr as *mut HeapAlloc<NonStatic>);
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
        // let taco = CLASSES[4];
        //
        // let class_file = ClassFile::parse_class(taco).unwrap();
        // let field = ClassFile::parse_class(taco).unwrap();
        //
        // let containing_class =
        // HeapAlloc::<NonStatic>::new(&Class::from(&class_file), 0);
        // let field_ref = HeapAlloc::<NonStatic>::new(&Class::from(&field), 0);
        //
        // unsafe {
        //     let heap = &mut *(containing_class as *mut HeapAlloc<NonStatic>);
        //     heap.set_field(
        //         "printWriter",
        //         Value::Ref(ptr::read(field_ref as *const
        // HeapAlloc<NonStatic>)),     )
        //     .unwrap();
        //
        //     HeapAlloc::deallocate(field_ref as *mut HeapAlloc<NonStatic>);
        //     HeapAlloc::deallocate(containing_class as *mut
        // HeapAlloc<NonStatic>); }
    }
}
