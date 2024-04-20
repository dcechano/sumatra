use sumatra_parser::{constant_pool::ConstantPool, method::Method};

use crate::value::Value;

//TODO rework Call frame to use manual allocation
// Check the requirements for where call frames should be allocated.
#[derive(Debug)]
pub(crate) struct CallFrame<'vm> {
    pub(crate) method: &'vm Method,
    pub(crate) pc: usize,
    pub(crate) locals: Vec<&'vm Value>,
    pub(crate) stack: Vec<Value>,
    pub(crate) cp: &'vm ConstantPool,
}
//
// impl<'vm> From<&'vm Method> for CallFrame<'vm> {
//     fn from(method: &'vm Method, cp: &'vm ConstantPool) -> Self {
//         CallFrame {
//             method,
//             pc: 0,
//             locals: vec![],
//             op_stack: vec![],
//             cp
//         }
//     }
// }

impl<'vm> CallFrame<'vm> {
    pub(crate) fn construct_cf(method: &'vm Method, cp: &'vm ConstantPool) -> Self {
        Self {
            method,
            pc: 0,
            locals: vec![],
            stack: vec![],
            cp,
        }
    }
}
