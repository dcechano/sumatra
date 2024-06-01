use crate::{
    alloc::{alloc_type::NonStatic, oop::HeapAlloc},
    class::Class,
    instance_data::InstanceData,
    reference_types::{ArrayRef, ObjRef},
};
use sumatra_parser::instruction::ArrayType;

pub(crate) struct Heap {
    gen1: Vec<*mut HeapAlloc<NonStatic>>,
    gen2: Vec<*mut HeapAlloc<NonStatic>>,
    tenured: Vec<*mut HeapAlloc<NonStatic>>,
}

impl Heap {
    pub(crate) fn new() -> Self {
        Self {
            gen1: Vec::with_capacity(128),
            gen2: Vec::with_capacity(128),
            tenured: Vec::with_capacity(128),
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
}
