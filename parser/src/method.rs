use crate::{attribute::Attribute, flags::MethodAccessFlags};

#[derive(Debug)]
pub struct Method {
    pub access_flags: MethodAccessFlags,
    pub name_index: usize,
    pub descriptor_index: usize,
    pub attributes: Vec<Attribute>,
}
