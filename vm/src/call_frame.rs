use anyhow::{bail, Result};
use sumatra_parser::{constant_pool::ConstantPool, method::Method};

use crate::{class::Class, value::Value};

#[derive(Debug)]
pub(crate) struct CallFrame {
    pub(crate) class: &'static Class,
    pub(crate) method: &'static Method,
    pub(crate) pc: usize,
    pub(crate) stack: Vec<Value>,
    pub(crate) locals: Vec<Value>,
    pub(crate) cp: &'static ConstantPool,
}

impl CallFrame {
    pub(crate) fn new(
        class: &'static Class,
        method: &'static Method,
        cp: &'static ConstantPool,
        locals: Vec<Value>,
    ) -> Self {
        debug_assert!(
            locals.len() <= method.code.max_locals as usize,
            "Locals array to long in method: {}. Expected max_locals: {},\
            Received: {}",
            method.name,
            method.code.max_locals,
            locals.len()
        );

        Self {
            class,
            method,
            pc: 0,
            locals,
            stack: vec![],
            cp,
        }
    }

    /// Returns a clone of the value at the top of the operand stack
    pub(crate) fn clone_top(&self) -> Value { self.stack.last().unwrap().clone() }

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

    /// Inserts `value` on offset entries from the top of the stack. `offset` is
    /// assumed to be > 0.
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

    pub(crate) fn push(&mut self, value: Value) {
        self.stack.push(value);
        let max_stack = self.method.code.max_stack as usize;
        if self.stack.len() > max_stack {
            panic!(
                "Stack overflowed the max_stack value: {max_stack}, current stack size: {}",
                self.stack.len()
            );
        }
    }

    pub(crate) fn pop(&mut self) -> Value { self.stack.pop().unwrap() }
}
