use std::collections::HashMap;

use anyhow::{bail, Result};

use crate::{
    alloc::{alloc_type::NonStatic, oop::HeapAlloc},
    class::Class,
    data_types::{
        array::{ArrayComp, ArrayRef},
        instance_data::InstanceData,
        object::ObjRef,
    },
    vm::{self, STRING, VM},
};

pub(crate) struct Heap {
    gen1: Vec<*mut HeapAlloc<NonStatic>>,
    gen2: Vec<*mut HeapAlloc<NonStatic>>,
    tenured: Vec<*mut HeapAlloc<NonStatic>>,
    classes: HashMap<String, *mut HeapAlloc<NonStatic>>,
    strings: HashMap<String, *const HeapAlloc<NonStatic>>,
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

    pub(crate) fn get_interned_str(&mut self, string: &str) -> Option<ObjRef> {
        match self.strings.get(string) {
            Some(obj) => {
                // SAFETY: Since we manage the pointer ourselves, we know it is valid
                // as long as the pointer wasn't invalidated elsewhere.
                Some(unsafe { ObjRef::from_raw(*obj as *mut _) })
            }
            None => None,
        }
    }

    pub(crate) fn intern_string(&mut self, rust_string: &str, java_string: ObjRef) -> Result<()> {
        if self.strings.contains_key(rust_string) {
            //TODO feels weird to error on an attempt to
            //intern a string. The angle is that the caller probably didn't mean to?
            bail!("Attempt to intern a string that already exists.");
        }

        self.strings
            .insert(rust_string.to_string(), java_string.get_inner());
        Ok(())
    }

    pub(crate) fn is_interned(&self, string: &str) -> bool { self.strings.contains_key(string) }

    pub(crate) fn new_class_object(
        &mut self,
        instance_class: &'static Class,
        java_lang_class: &'static Class,
        java_lang_object: &'static Class,
    ) -> ObjRef {
        let super_class = if instance_class.super_class == 0 {
            vec![]
        } else {
            vec![java_lang_object]
        };

        let obj = ObjRef::new(InstanceData::new(
            java_lang_class,
            vm::CLASS_CLASS_ID,
            super_class,
        ));
        let class_name = instance_class.get_name();
        debug_assert!(!self.classes.contains_key(&class_name));
        self.classes.insert(class_name, obj.get_inner() as *mut _);
        obj
    }

    /// Returns the java.lang.Class object for the class represented by
    /// `class_name`.
    pub(crate) fn get_class_obj(&self, class_name: &str) -> ObjRef {
        // SAFETY: Since we manage the pointer ourselves, we know it is valid
        // as long as the pointer wasn't invalidated elsewhere.
        unsafe { ObjRef::from_raw(*self.classes.get(class_name).unwrap()) }
    }

    /// Retrieves the name of the class associated with the passed in java ref.
    pub(crate) fn class_name(&self, obj: ObjRef) -> String {
        for (name, entry) in self.classes.iter() {
            if obj.get_inner() == (*entry as *const _) {
                return name.to_string();
            }
        }
        panic!("No java.lang.Class instance found for obj in heap::class_name");
    }
}
