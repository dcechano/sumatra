use crate::{
    data_types::{object::ObjRef, value::Value},
    vm::VM,
};
use anyhow::Result;

pub type NativeMethod = fn(&mut VM, Option<ObjRef>, Vec<Value>) -> Result<Option<Value>>;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct NativeIdentifier {
    class: String,
    method: String,
}

impl NativeIdentifier {
    pub fn new(class: String, method: String) -> Self { Self { class, method } }
}
