#![allow(warnings)]
// TODO Remove

use crate::{class::Class, vm::VM};
use anyhow::{bail, Result};
use sumatra_parser::constant::Constant;

pub mod alloc;
mod call_frame;
pub mod class;
mod compare;
pub mod data_types;
pub mod lli;
pub mod native_registry;
pub mod result;
pub mod vm;

pub use native_registry as native;

/// Checks if `superclass` is the superclass of `child_class`.
/// By the logic of this function a class is the superclass of itself. Will
/// return `Ok(true)` if `child_class` == `superclass`.
fn is_subclass(
    vm: &mut VM,
    child_class: &'static Class,
    superclass: &'static Class,
) -> Result<bool> {
    if "java/lang/Object" == superclass.get_name() {
        return Ok(true);
    }

    if child_class.super_class == 0 {
        return Ok(false);
    }

    let mut class = child_class;
    loop {
        if class.get_name() == superclass.get_name() {
            return Ok(true);
        }
        class = get_superclass(vm, class).unwrap();
        if class.super_class == 0 {
            return Ok(false);
        }
    }
}

/// Checks if the `implementor` implements `interface`.
fn is_implemented(
    vm: &mut VM,
    implementor: &'static Class,
    interface: &'static Class,
) -> Result<bool> {
    if !interface.is_interface() {
        panic!("Expected an interface.");
    }

    // self.super_class == 0 means that self is java/lang/Object
    if implementor.super_class == 0 {
        return Ok(false);
    }
    let interface_name = interface.get_name();
    let mut class = implementor;
    loop {
        if class.interfaces().contains(&interface_name) {
            return Ok(true);
        }
        class = get_superclass(vm, class).unwrap();
        if class.super_class == 0 {
            return Ok(false);
        }
    }
}

/// Gets the superclass of the provided class. It is the responsibility of the
/// caller to ensure that `class` is not java/lang/Object.
fn get_superclass(vm: &mut VM, class: &'static Class) -> Result<&'static Class> {
    let super_index = class.super_class;
    let Constant::Class(name_index) = class.cp.get(super_index).unwrap() else {
        bail!("Expected a Constant::Class for index.");
    };

    let super_name = class.cp.get_utf8(*name_index).unwrap();
    vm.load_class_immut(super_name)
}
