use crate::{
    attribute::RuntimeAnnotation, constant::Constant, desc_types::FieldDescriptor,
    flags::FieldAccessFlags,
};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Field {
    pub access_flags: FieldAccessFlags,
    pub name: String,
    pub descriptor: String,
    pub parsed_descriptor: FieldDescriptor,
    pub signature: String,
    pub constant_value: Constant,
    pub synthetic: bool,
    pub deprecated: bool,
    pub runtime_annotations: Vec<RuntimeAnnotation>,
}
