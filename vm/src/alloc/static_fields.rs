use anyhow::Result;

use crate::{
    alloc::{alloc_type::Static, oop::HeapAlloc},
    class::Class,
    data_types::value::Value,
};

#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct StaticFields {
    alloc: HeapAlloc<Static>,
}

impl StaticFields {
    pub(super) fn new(class: &Class, class_id: usize) -> Self {
        let alloc = HeapAlloc::<Static>::new(&class, class_id);
        Self { alloc }
    }

    pub(crate) fn get_field(&self, name: &str) -> Result<&'static Value> {
        self.alloc.get_field(name)
    }

    pub(crate) fn get_field_mut(&mut self, name: &str) -> Result<&'static mut Value> {
        self.alloc.get_field_mut(name)
    }

    pub(crate) fn set_field(&mut self, name: &str, data: Value) -> Result<()> {
        self.alloc.set_field(name, data)
    }
}
