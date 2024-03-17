/// All possible `constant_pool` entries.
#[derive(Debug)]
pub enum Constant {
    Dummy,
    UTF8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class {
        name_index: usize,
    },
    String {
        string_index: usize,
    },
    FieldRef {
        class_index: usize,
        name_and_type_index: usize,
    },
    MethodRef {
        class_index: usize,
        name_and_type_index: usize,
    },
    InterfaceMethodRef {
        class_index: usize,
        name_and_type_index: usize,
    },
    NameAndType {
        name_index: usize,
        descriptor_index: usize,
    },
    MethodHandle {
        reference_kind: u8,
        reference_index: usize,
    },
    MethodType {
        descriptor_index: usize,
    },
    Dynamic {
        bootstrap_method_attr_index: usize,
        name_and_type_index: usize,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: usize,
        name_and_type_index: usize,
    },
    Module {
        name_index: usize,
    },
    Package {
        name_index: usize,
    },
}
