/// All possible `constant_pool` entries.
#[derive(Debug, Default, PartialEq, Clone)]
pub enum Constant {
    #[default]
    Dummy,
    /// A UTF8 constant
    UTF8(String),
    /// A Java `int`
    Integer(i32),
    /// A Java `float`
    Float(f32),
    /// A Java `long`
    Long(i64),
    /// A Java `double`
    Double(f64),
    /// A Java Class. Stored value is the index of the name.
    Class(usize),
    /// A Java `String`.
    String(usize),
    /// Metadata for a class field.
    FieldRef {
        class_index: usize,
        name_and_type_index: usize,
    },
    /// Metadata for a class method.
    MethodRef {
        class_index: usize,
        name_and_type_index: usize,
    },
    /// Metadata for an interface method.
    InterfaceMethodRef {
        class_index: usize,
        name_and_type_index: usize,
    },
    /// Used to represent a field or method,
    /// without indicating which class or interface type it belongs to.
    /// The index to the `Class` is held in the constant that pointed to this
    /// `NameAndType`. ## Fields
    /// `name_index` is the index of the UTF8 representation of its name.
    /// `descriptor_index` is the index of the UT8 representation if its
    /// descriptor.
    NameAndType {
        name_index: usize,
        descriptor_index: usize,
    },
    /// A method handle.
    /// ## Fields
    /// The value of the `reference_kind` item must be in the range 1 to 9.
    /// The `reference_kind` denotes the kind of this method handle,
    /// which characterizes its bytecode behavior.
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-5.html#jvms-5.4.3.5
    ///
    /// `reference_index` is an index into the constant pool. The value
    /// the index points to depends on the value of `reference_kind`.
    MethodHandle {
        reference_kind: u8,
        reference_index: usize,
    },
    /// A method type. The `usize` is the index of the UTF8 method descriptor in
    /// the constant pool.
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
