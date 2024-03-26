// TODO elaborate with documentation from spec
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Annotation {
    pub type_index: usize,
    pub value_pairs: Vec<ElementPairs>,
}
// TODO elaborate with documentation from spec
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ElementPairs(pub usize, pub ElementValue);

// TODO elaborate with documentation from spec
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ElementValue {
    ConstIndex(usize),
    EnumConst(usize, usize),
    ClassIndex(usize),
    AnnotationValue(Annotation),
    Array(Vec<ElementValue>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypeAnnotation {
    pub type_target: TypeTarget,
    pub type_path: TypePath,
    pub type_index: usize,
    pub value_pairs: Vec<ElementPairs>,
}

// TODO The arguments may be unnecessary but different target
// types meaning different things in different contexts despite
// mapping to the same variant.
// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.20
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TypeTarget {
    TypeParameter {
        value: u8,
        type_param_index: usize,
    },
    SuperType {
        value: u8,
        supertype_index: usize,
    },
    TypeParameterBound {
        value: u8,
        type_parameter_index: usize,
        bound_index: usize,
    },
    Empty(u8),
    FormalParameter {
        value: u8,
        formal_param_index: usize,
    },
    Throws {
        value: u8,
        throws_type_index: usize,
    },
    LocalVar {
        value: u8,
        target_table: LocalVarTargetTable,
    },
    Catch {
        value: u8,
        exception_table_index: usize,
    },
    Offset {
        value: u8,
        offset: u16,
    },
    TypeArgument {
        value: u8,
        type_arg_index: usize,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LocalVarTargetTable(pub Vec<LocalVarTargetTableEntry>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LocalVarTargetTableEntry {
    pub start_pc: u16,
    pub length: u16,
    pub index: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypePath(pub Vec<TypePathEntry>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypePathEntry {
    pub type_path_kind: u8,
    pub type_arg_index: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ParameterAnnotation(pub Vec<Annotation>);
