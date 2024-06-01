use std::mem;

use crate::data_types::value::Value;

pub mod alloc_type;
pub mod fields_table;
pub mod header;
pub mod heap;
pub mod method_area;
pub mod oop;
pub mod static_fields;

const VALUE_SIZE: usize = mem::size_of::<Value>();
const VALUE_ALIGN: usize = mem::align_of::<Value>();
