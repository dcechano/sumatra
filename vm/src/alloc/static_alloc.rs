use crate::{
    alloc::{alloc_type::Static, oop::HeapAlloc},
    class::Class,
    value::Value,
};

#[derive(Debug)]
pub(crate) struct StaticAlloc {
    class: Class,
    alloc: HeapAlloc<Static>,
}

impl StaticAlloc {
    pub(crate) fn new(class: Class, index: usize) -> Self {
        let alloc = HeapAlloc::<Static>::new(&class, index);
        Self { class, alloc }
    }

    pub(crate) fn get_class(&self) -> &Class { &self.class }

    pub(crate) fn get_field(&self, name: &str) -> &'static Value {
        self.alloc.get_field(name).unwrap()
    }

    pub(crate) fn get_field_mut(&mut self, name: &str) -> &'static mut Value {
        self.alloc.get_field_mut(name).unwrap()
    }
}
