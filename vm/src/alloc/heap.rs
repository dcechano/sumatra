use std::collections::HashMap;

use crate::{
    alloc::{alloc_type::NonStatic, oop::HeapAlloc},
    class::Class,
    data_types::{
        array::{ArrayComp, ArrayRef},
        instance_data::InstanceData,
        object::ObjRef,
    },
};

pub(crate) struct Heap {
    gen1: Vec<*mut HeapAlloc<NonStatic>>,
    gen2: Vec<*mut HeapAlloc<NonStatic>>,
    tenured: Vec<*mut HeapAlloc<NonStatic>>,
    classes: HashMap<String, *mut HeapAlloc<NonStatic>>,
    strings: HashMap<String, *mut HeapAlloc<NonStatic>>,
}

impl Heap {
    pub(crate) fn new() -> Self {
        Self {
            gen1: Vec::with_capacity(128),
            gen2: Vec::with_capacity(128),
            tenured: Vec::with_capacity(128),
            classes: HashMap::with_capacity(128),
            strings: HashMap::with_capacity(128),
        }
    }

    pub(crate) fn new_array(&mut self, length: usize, array_comp: ArrayComp) -> ArrayRef {
        let array = ArrayRef::new(length, array_comp);
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

    pub(crate) fn interned_string(&mut self, string: &str) -> Option<ObjRef> {
        match self.strings.get(string) {
            Some(obj) => {
                // SAFETY: Since we manage the pointer ourselves, we know it is valid
                // as long as the pointer wasn't invalidated elsewhere.
                Some(unsafe { ObjRef::from_raw(*obj) })
            }
            None => None,
        }
    }

    pub(crate) fn is_interned(&self, string: &str) -> bool { self.strings.contains_key(string) }

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
        //TODO this line is wrong. The class_id param is not the class id for
        // java_lang_class, as the method expects it to be. It is the class id
        // for the instance_class param. I am not sure if this was an oversight
        // or there is something I forgot. I will have to come back to this.
        let obj = ObjRef::new(InstanceData::new(java_lang_class, class_id, super_class));
        self.classes
            .insert(instance_class.get_name(), obj.get_inner() as *mut _);
        obj
    }

    /// Returns the java.lang.Class object for the class represented by
    /// `class_name`.
    pub(crate) fn get_class_obj(&self, class_name: &str) -> ObjRef {
        // SAFETY: Since we manage the pointer ourselves, we know it is valid
        // as long as the pointer wasn't invalidated elsewhere.
        unsafe { ObjRef::from_raw(*self.classes.get(class_name).unwrap()) }
    }
}
