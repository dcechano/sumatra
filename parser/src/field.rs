use crate::{
    attribute::RuntimeAnnotation, constant::Constant, desc_types::FieldDescriptor,
    flags::FieldAccessFlags,
};
use crate::desc_types::MethodDescriptor;

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

impl Field {
    pub fn is_static(&self) -> bool {
        self.access_flags.contains(FieldAccessFlags::STATIC)
    }
    
    pub fn is_final(&self) -> bool {
        self.access_flags.contains(FieldAccessFlags::FINAL)
    }

    pub fn get_parsed_descriptor(&self) -> FieldDescriptor {
        self.parsed_descriptor.clone()
    }
}
