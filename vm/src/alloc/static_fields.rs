use crate::{
    alloc::{alloc_type::Static, oop::HeapAlloc},
    class::Class,
};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct StaticFields {
    alloc: HeapAlloc<Static>,
}

impl StaticFields {
    pub(crate) fn new(class: &Class, index: usize) -> Self {
        let alloc = HeapAlloc::<Static>::new(&class, index);
        Self { alloc }
    }
}

impl Deref for StaticFields {
    type Target = HeapAlloc<Static>;

    fn deref(&self) -> &Self::Target { &self.alloc }
}

impl DerefMut for StaticFields {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.alloc }
}
