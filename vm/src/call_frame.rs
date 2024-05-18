use anyhow::{bail, Result};
use sumatra_parser::{constant_pool::ConstantPool, method::Method};

use crate::value::Value;

#[derive(Debug)]
pub(crate) struct CallFrame {
    pub(crate) method: &'static Method,
    pub(crate) pc: usize,
    pub(crate) num_locals: usize,
    pub(crate) stack: Vec<Value>,
    pub(crate) locals: Vec<Value>,
    pub(crate) cp: &'static ConstantPool,
}

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
    
    /// Returns a clone of the value at the top of the operand stack
    pub(crate) fn clone_top(&self) -> Value {
        self.stack.last().unwrap().clone()
    }

    pub(crate) fn insert_local(&mut self, index: usize, value: Value) -> Result<()> {
        match self.locals.get_mut(index) {
            None => bail!("Invalid index into the locals array."),
            Some(local) => *local = value,
        };
        Ok(())
    }

    /// Returns a local variable for mutating.
    pub(crate) fn get_local(&mut self, index: usize) -> &mut Value {
        self.locals.get_mut(index).expect(&format!(
            "{index} was not a valid index into the locals array."
        ))
    }
    
    /// Inserts `value` on offset entries from the top of the stack. `offset` is assumed
    /// to be > 0.
    pub(crate) fn insert(&mut self, offset: usize, value: Value) {
        let index = self.stack.len() - offset - 1;
        self.stack.insert(index, value)
    }

    /// Retrieves a value from the local variable array.
    pub(crate) fn load(&self, index: usize) -> Result<Value> {
        match self.locals.get(index) {
            None => bail!("Invalid index into the locals array."),
            Some(value) => Ok(value.clone()),
        }
    }

    pub(crate) fn push(&mut self, value: Value) { self.stack.push(value) }

    pub(crate) fn pop(&mut self) -> Value { self.stack.pop().unwrap() }
}
