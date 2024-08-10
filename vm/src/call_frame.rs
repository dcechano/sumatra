use anyhow::{bail, Result};
use std::collections::VecDeque;

use sumatra_parser::{constant_pool::ConstantPool, method::Method};

use crate::{
    class::Class,
    data_types::{value, value::Value},
};

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

    /// Push value to the operand stack. If value is a double or long,
    /// the value will be cloned and pushed twice since doubles and longs
    /// take 2 spots on the operand stack.
    pub(crate) fn push(&mut self, value: Value) {
        if matches!(value, (Value::Long(_) | Value::Double(_))) {
            self.stack.push(value.clone());
        }
        self.stack.push(value);
        let max_stack = self.method.code.max_stack as usize;
        if self.stack.len() > max_stack {
            panic!(
                "Stack overflowed. The max_stack value: {max_stack}, current stack size: {}",
                self.stack.len()
            );
        }
    }

    /// Pop the top value off the operand stack. If the value is a double or
    /// long, 2 values will be popped since doubles and longs take 2 spots
    /// on the stack.
    pub(crate) fn pop(&mut self) -> Value {
        let value = self.stack.pop().unwrap();
        if matches!(value, (Value::Long(_) | Value::Double(_))) {
            let _ = self
                .stack
                .pop()
                .expect("There was not a second double or long on stack.");
        }
        value
    }

    /// Pop `num_params` elements from the operand stack. This method is used to
    /// construct a new call frame where the arguments for the new call
    /// frame are stored on this call frame's operand stack. Longs and
    /// doubles are considered to be 2 parameters per the JVM spec.
    fn pop_params(&mut self, num_params: usize) -> Vec<Value> {
        // Cant use .map() the rev() here because map is called lazily and calling rev()
        // after map won't fix the order of the vec. So we do this...
        let mut deque = VecDeque::with_capacity(num_params);
        (0..num_params).for_each(|_| {
            deque.push_front(self.stack.pop().expect("Not enough params in pop_params."))
        });
        deque.into()
    }

    /// Populate the locals array using the array of initialized method
    /// parameters. It is assumed that the `num_locals` is greater than or
    /// equal to the `num_params` since the JVM spec considers method
    /// parameters to be locals. In other words the `num_locals` includes the
    /// `num_params`.
    pub(crate) fn populate_locals(&mut self, num_locals: usize, num_params: usize) -> Vec<Value> {
        if num_params > num_locals {
            panic!("The number of locals cannot be the greater than the number of params.");
        }

        let mut locals = self.pop_params(num_params);
        let num_dummies = num_locals - num_params;
        locals.extend(vec![Value::Null; num_dummies]);
        locals
    }
}
