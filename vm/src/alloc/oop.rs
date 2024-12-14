use std::{
    alloc,
    alloc::{handle_alloc_error, Layout},
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr,
};

use sumatra_parser::{field::Field, flags::FieldAccessFlags, instruction::ArrayType};

use crate::{
    alloc::{
        alloc_type::{AllocType, NonStatic, Static},
        header::Header,
    },
    class::Class,
    data_types::{array::ArrayComp, value::Value},
    result::{Error, Result},
    vm::VM,
    vm_error,
};

pub struct HeapAlloc<T: AllocType> {
    pub header: Header,
    pub fields: *mut Value,
    pub elements: *mut Value,
    _phantom: PhantomData<Value>,
    _static: PhantomData<T>,
}

impl<T: AllocType> HeapAlloc<T> {
    /// Returns the class_id for the given `HeapAlloc`.
    /// SAFETY: raw pointer to `HeapAlloc` must be valid by all
    /// rules outlined here: https://doc.rust-lang.org/std/ptr/index.html#safety  
    pub(crate) unsafe fn get_class_id(obj: *const HeapAlloc<T>) -> usize { (*obj).header.class_id }

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
    fn get_field_inner(&self, name: &str) -> Result<*mut Value> {
        let offset = match self.header.fields.get(name) {
            None => {
                vm_error!("No field with name: {name}");
            }
            Some(offset) => offset,
        };
        // SAFETY: offset is valid due to the offset being calculated from the
        // memory region itself, so the offset always points into valid memory.
        unsafe { Ok(self.fields.add(*offset)) }
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

        if !(*heap).elements.is_null() {
            let (length, _) = (*heap).header.array_data.as_ref().unwrap();
            let layout = Layout::array::<Value>(*length).unwrap();
            alloc::dealloc((*heap).elements as *mut u8, layout);
        }

        Self::dealloc_data(heap);

        ptr::drop_in_place(&mut (*heap).header as *mut Header);

        alloc::dealloc(heap as *mut u8, Layout::new::<HeapAlloc<NonStatic>>());
    }

    // So HeapAllocs can be created for testing and deallocated properly.
    #[cfg(test)]
    pub(crate) unsafe fn dealloc_test_obj(heap: *mut HeapAlloc<T>) { Self::deallocate(heap); }

    #[inline]
    unsafe fn dealloc_data(heap: *mut HeapAlloc<T>) {
        let data = (*heap).fields;
        if !data.is_null() {
            // We need not worry about fields.len == 0 because the only way the ptr
            // is not null is that there were fields to justify the initial allocation.
            let size = (*heap).header.fields.len();
            debug_assert!(size != 0);
            let layout = Layout::array::<Value>(size).unwrap();
            alloc::dealloc(data as *mut u8, layout);
        }
    }
}

impl HeapAlloc<Static> {
    // Even though we are returning an owned HeapAlloc here, there is no risk of
    // deallocation because the only caller is the MethodArea (via
    // StaticFields::new()). The MethodArea immediately stores the HeapAlloc in
    // static memory and deallocation happens when the VM is dropped.
    #[inline]
    pub(super) fn new(class: &Class, class_id: usize) -> HeapAlloc<Static> {
        // A HeapAlloc<Static> does not need access to the instance fields of its
        // superclasses, hence the empty vec.
        let (header, data) = Header::new::<Static>(class, class_id, vec![]);
        HeapAlloc {
            header,
            fields: data,
            elements: ptr::null_mut(),
            _phantom: Default::default(),
            _static: Default::default(),
        }
    }
}

impl HeapAlloc<NonStatic> {
    #[allow(clippy::new_ret_no_self)]
    #[inline]
    pub(crate) fn new(
        class: &Class,
        class_id: usize,
        super_class: Vec<&'static Class>,
    ) -> *mut HeapAlloc<NonStatic> {
        // SAFETY: `Layout::new::<HeapAlloc>())` is valid so alloc is safe.
        let ptr = unsafe {
            alloc::alloc(Layout::new::<HeapAlloc<NonStatic>>()) as *mut HeapAlloc<NonStatic>
        };
        if ptr.is_null() {
            handle_alloc_error(Layout::new::<HeapAlloc<NonStatic>>())
        }

        let (header, fields) = Header::new::<NonStatic>(class, class_id, super_class);
        // SAFETY: ptr is valid for writes since we asserted nonnull above.
        unsafe {
            ptr::write(
                ptr,
                HeapAlloc {
                    header,
                    fields,
                    elements: ptr::null_mut(),
                    _phantom: Default::default(),
                    _static: Default::default(),
                },
            );
            ptr
        }
    }

    pub(crate) fn new_array(length: usize, array_type: ArrayComp) -> *mut HeapAlloc<NonStatic> {
        if length > isize::MAX as usize {
            panic!("Attempted to initialize array with illegal length: {length}");
        }
        let ptr = unsafe {
            alloc::alloc(Layout::new::<HeapAlloc<NonStatic>>()) as *mut HeapAlloc<NonStatic>
        };
        if ptr.is_null() {
            handle_alloc_error(Layout::new::<HeapAlloc<NonStatic>>())
        }

        let elements = match length == 0 {
            true => ptr::null_mut(),
            // SAFETY: length has been verified to be greater than 0 and less than isize::MAX.
            false => unsafe {
                let elements = alloc::alloc(Layout::array::<Value>(length).unwrap()) as *mut Value;
                if elements.is_null() {
                    handle_alloc_error(Layout::new::<HeapAlloc<NonStatic>>())
                }
                Self::default_values(length, elements, &array_type);
                elements
            },
        };

        let header = Header::new_array(length, array_type);

        unsafe {
            ptr::write(
                ptr,
                HeapAlloc {
                    header,
                    fields: ptr::null_mut(),
                    elements,
                    _phantom: Default::default(),
                    _static: Default::default(),
                },
            );
            ptr
        }
    }

    fn default_values(length: usize, elements: *mut Value, array_type: &ArrayComp) {
        let default = Value::from(array_type.clone());
        for i in 0..length {
            // SAFETY: The responsibility for length being a valid index is left to the
            // caller.
            unsafe { ptr::write(elements.add(i), default.clone()) }
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
                let ptr = self.fields.add(*offset) as *const Value;
                writeln!(f, "{name} {:?}", *ptr)?;
            }
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod test {
    use std::{fmt::Debug, ptr};

    use sumatra_parser::{class_file::ClassFile, flags::FieldAccessFlags, instruction::ArrayType};

    use crate::{
        alloc::oop::{HeapAlloc, NonStatic},
        class::Class,
        data_types::{array::ArrayComp, value::Value},
    };

    //TODO fine for now but eventually this will have to removed and made to use
    // relative paths so that tests pass in any environment.
    const CLASSES: [&str; 5] = [
        "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/Main.class",
        "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/Interface.class",
        "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/Import.class",
        "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/Simple.class",
        "/home/dylan/Documents/RustProjects/sumatra/java/out/production/sumatra/files/Taco.class",
    ];

    #[test]
    fn alloc() {
        for class in CLASSES {
            eprintln!("{class}");
            let class_file = ClassFile::parse_class(class).unwrap();
            let ptr = HeapAlloc::<NonStatic>::new(&Class::from(&class_file), 0, vec![]);
            unsafe {
                let heap = &mut *(ptr);
                for field in class_file.fields {
                    if !field.access_flags.contains(FieldAccessFlags::STATIC) {
                        heap.set_field(&field.name, Value::Long(69420)).unwrap();
                    }
                }

                HeapAlloc::deallocate(ptr);
            }
        }
    }

    #[test]
    #[cfg(miri)]
    fn no_leak_array() {
        unsafe {
            let ptr = HeapAlloc::new_array(10, ArrayComp::Int);
            HeapAlloc::deallocate(ptr)
        }
    }

    #[test]
    fn create_array() {
        const LENGTH: usize = 10;
        const CHANGE_INDEX: usize = 5;
        const NEW_ENTRY: Value = Value::Int(42);

        unsafe {
            let ptr = HeapAlloc::new_array(LENGTH, ArrayComp::Int);
            let (length, array_type) = (*ptr).header.array_data.as_ref().unwrap();
            assert_eq!(*length, LENGTH);
            assert_eq!(*array_type, ArrayComp::Int);
            for i in 0..LENGTH {
                assert_eq!((*ptr).elements.add(i).as_ref().unwrap(), &Value::Int(0));
            }
            ptr::write((*ptr).elements.add(CHANGE_INDEX), NEW_ENTRY.clone());
            assert_eq!(
                (*ptr).elements.add(CHANGE_INDEX).as_ref().unwrap(),
                &NEW_ENTRY
            );
            HeapAlloc::deallocate(ptr);
        }
    }
}
