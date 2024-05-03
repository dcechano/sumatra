use anyhow::{bail, Result};
use sumatra_parser::{constant_pool::ConstantPool, method::Method};

use crate::value::Value;

//TODO rework Call frame to use manual allocation
// Check the requirements for where call frames should be allocated.
#[derive(Debug)]
pub(crate) struct CallFrame {
    pub(crate) method: &'static Method,
    pub(crate) pc: usize,
    pub(crate) num_locals: usize,
    pub(crate) stack: Vec<Value>,
    pub(crate) locals: Vec<Value>,
    pub(crate) cp: &'static ConstantPool,
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

impl CallFrame {
    pub(crate) fn new(
        method: &'static Method,
        cp: &'static ConstantPool,
        num_locals: usize,
        locals: Vec<Value>,
    ) -> Self {
        Self {
            method,
            pc: 0,
            num_locals,
            locals,
            stack: vec![],
            cp,
        }
    }

    pub(crate) fn insert_local(&mut self, index: usize, value: Value) -> Result<()> {
        match self.locals.get_mut(index) {
            None => bail!("Invalid index into the locals array."),
            Some(local) => *local = value,
        };
        Ok(())
    }

    /// Retrieves a value from the local variable array.
    pub(crate) fn load(&self, index: usize) -> Result<Value> {
        match self.locals.get(index) {
            None => bail!("Invalid index into the locals array."),
            Some(value) => Ok(value.clone()),
        }
    }
    
    pub(crate) fn push(&mut self, value: Value) {
        self.stack.push(value)
    }
    
    pub(crate) fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }
}
