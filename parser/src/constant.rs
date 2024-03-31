/// All possible `constant_pool` entries.
#[derive(Debug, Default, PartialEq, Clone)]
pub enum Constant {
    #[default]
    Dummy,
    UTF8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class(usize),
    String(usize),
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
    MethodType(usize),
    Dynamic {
        bootstrap_method_attr_index: usize,
        name_and_type_index: usize,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: usize,
        name_and_type_index: usize,
    },
    Module(usize),
    Package(usize),
}
