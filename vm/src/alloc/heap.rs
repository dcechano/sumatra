use std::collections::HashMap;

use sumatra_parser::instruction::ArrayType;

use crate::{
    alloc::{alloc_type::NonStatic, oop::HeapAlloc},
    class::Class,
    data_types::{
        instance_data::InstanceData,
        reference_types::{ArrayRef, ObjRef},
    },
};

pub(crate) struct Heap {
    gen1: Vec<*mut HeapAlloc<NonStatic>>,
    gen2: Vec<*mut HeapAlloc<NonStatic>>,
    tenured: Vec<*mut HeapAlloc<NonStatic>>,
    classes: HashMap<String, *mut HeapAlloc<NonStatic>>,
}

impl Heap {
    pub(crate) fn new() -> Self {
        Self {
            gen1: Vec::with_capacity(128),
            gen2: Vec::with_capacity(128),
            tenured: Vec::with_capacity(128),
            classes: HashMap::with_capacity(128),
        }
    }

    pub(crate) fn new_array(&mut self, length: usize, array_type: ArrayType) -> ArrayRef {
        let array = ArrayRef::new(length, array_type);
        self.gen1.push(array.get_inner() as *mut _);
        array
    }

    pub(crate) fn new_object(&mut self, instance_data: InstanceData) -> ObjRef {
        let obj = ObjRef::new(instance_data);
        self.gen1.push(obj.get_inner() as *mut _);
        obj
    }

    pub(crate) fn new_tenured_obj(&mut self, instance_data: InstanceData) -> ObjRef {
        let obj = ObjRef::new(instance_data);
        self.tenured.push(obj.get_inner() as *mut _);
        obj
    }

    pub(crate) fn new_class_object(
        &mut self,
        instance_class: &'static Class,
        class_id: usize,
        java_lang_class: &'static Class,
        java_lang_object: &'static Class,
    ) -> ObjRef {
        let super_class = if class_id == 0 {
            vec![]
        } else {
            vec![java_lang_object]
        };
        let obj = ObjRef::new(InstanceData::new(java_lang_class, class_id, super_class));
        self.classes
            .insert(instance_class.get_name(), obj.get_inner() as *mut _);
        obj
    }

    /// Returns the java.lang.Class object for the class represented by
    /// `class_name`.
    pub(crate) fn get_class_obj(&self, class_name: &str) -> ObjRef {
        // SAFETY: Since we manage the pointer ourselves, we know it is valid,
        // as long as the pointer wasn't invalidated elsewhere.
        unsafe { ObjRef::from_raw(*self.classes.get(class_name).unwrap()) }
    }
}
