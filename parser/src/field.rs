use crate::{attribute::RuntimeAnnotation, constant::Constant, flags::FieldAccessFlags};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Field {
    pub access_flags: FieldAccessFlags,
    pub name: String,
    pub descriptor: String,
    pub signature: String,
    pub constant_value: Constant,
    pub synthetic: bool,
    pub deprecated: bool,
    pub runtime_annotations: Vec<RuntimeAnnotation>,
}
