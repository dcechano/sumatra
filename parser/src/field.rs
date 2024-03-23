use crate::{attribute::RuntimeAnnotation, constant::Constant, flags::FieldAccessFlags};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Field {
    pub access_flags: FieldAccessFlags,
    pub name_index: usize,
    pub descriptor_index: usize,
    pub signature_index: usize,
    pub constant_value: Constant,
    pub synthetic: bool,
    pub deprecated: bool,
    pub runtime_annotations: Vec<RuntimeAnnotation>,
}
