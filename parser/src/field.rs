use crate::{attribute::Attribute, flags::FieldAccessFlags};

#[derive(Debug)]
pub struct Field {
    pub access_flags: FieldAccessFlags,
    pub name_index: usize,
    pub descriptor_index: usize,
    pub attributes: Vec<Attribute>,
}
