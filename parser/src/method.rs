use crate::{
    attribute::{Attribute, Code, Exceptions, RuntimeAnnotation},
    flags::MethodAccessFlags,
};

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Method {
    pub access_flags: MethodAccessFlags,
    pub name: String,
    pub descriptor: String,
    pub signature: String,
    pub code: Code,
    pub exceptions: Exceptions,
    pub runtime_annotations: Vec<RuntimeAnnotation>,
    pub attributes: Vec<Attribute>,
}
