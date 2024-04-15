use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    ops::{Deref, DerefMut},
};

#[derive(Default)]
#[repr(transparent)]
pub(crate) struct FieldsTable(HashMap<String, usize>);

impl FieldsTable {
    #[inline]
    pub(crate) fn with_capacity(cap: usize) -> Self { Self(HashMap::with_capacity(cap)) }
}

impl Deref for FieldsTable {
    type Target = HashMap<String, usize>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for FieldsTable {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Debug for FieldsTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { writeln!(f, "{:#?}", &self.0) }
}
