use crate::{
    attribute::{Attribute, Code, Exceptions, RuntimeAnnotation},
    desc_types::MethodDescriptor,
    flags::MethodAccessFlags,
};

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Method {
    pub access_flags: MethodAccessFlags,
    pub name: String,
    pub descriptor: String,
    pub parsed_descriptor: MethodDescriptor,
    pub signature: String,
    pub code: Code,
    pub exceptions: Exceptions,
    pub synthetic: bool,
    pub deprecated: bool,
    pub runtime_annotations: Vec<RuntimeAnnotation>,
    pub attributes: Vec<Attribute>,
}
