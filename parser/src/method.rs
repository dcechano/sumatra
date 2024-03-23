use crate::{
    attribute::{Attribute, Code, Exceptions, RuntimeAnnotation},
    flags::MethodAccessFlags,
};

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Method {
    pub access_flags: MethodAccessFlags,
    pub name_index: usize,
    pub descriptor_index: usize,
    pub signature_index: usize,
    pub code: Code,
    pub exceptions: Exceptions,
    pub runtime_annotations: Vec<RuntimeAnnotation>,
    pub attributes: Vec<Attribute>,
}
