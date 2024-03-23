use std::ops::{Deref, DerefMut};

use anyhow::{bail, Result};

use crate::constant::{Constant, Constant::UTF8};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ConstantPool(Vec<Constant>);

impl ConstantPool {
    pub fn new(capacity: usize) -> Self { Self(Vec::with_capacity(capacity)) }

    pub(crate) fn get_utf8(&self, index: usize) -> Result<&str> {
        match self.get(index) {
            Some(string) => {
                if let UTF8(string) = string {
                    return Ok(&string);
                }
            }
            _ => {}
        }
        bail!("Unable to retrieve string at index {index}");
    }
}

impl Deref for ConstantPool {
    type Target = Vec<Constant>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for ConstantPool {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
